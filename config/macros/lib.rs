//! Macros for config declarations
//! ==============================
//!
//! This crate defines procedural macros for graceful `CVar` declarations.
//!
//! These macros are reexported from the root config crate as [`config::config`]
//! and [`config::config_registrar`] -- that's generally where you'll want to
//! invoke them from.

use proc_macro::TokenStream as RustcTokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{parse_macro_input, Attribute, Expr, Ident, Token, Type, Visibility};

struct ConfigDeclaration {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    ty: Type,
    default_value: Expr,
}
impl Parse for ConfigDeclaration {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        // Config entry format:
        // /** docstring */
        // IDENTIFIER: type = default;
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let default_value = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(ConfigDeclaration {
            attrs,
            visibility,
            name,
            ty,
            default_value,
        })
    }
}

struct ConfigBlock {
    declarations: Vec<ConfigDeclaration>,
}
impl Parse for ConfigBlock {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        let mut acc = Vec::new();
        while !input.is_empty() {
            let cfg = input.parse()?;
            acc.push(cfg);
        }
        Ok(ConfigBlock { declarations: acc })
    }
}
impl std::ops::Deref for ConfigBlock {
    type Target = Vec<ConfigDeclaration>;

    fn deref(&self) -> &Self::Target {
        &self.declarations
    }
}

#[proc_macro]
/// Create storage, registration, and premain initialization for a block of
/// configuration variables.
///
/// For usage, see the main `config` crate's docs.
pub fn config(input: RustcTokenStream) -> RustcTokenStream {
    let configs = parse_macro_input!(input as ConfigBlock);

    // Linting pass: run completely, first, so if anything fails to generate
    // we at least can still give feedback on the whole block
    for ConfigDeclaration { attrs, name, .. } in configs.iter() {
        let purpose = attrs.iter().find_map(|a| {
            let meta = a.parse_meta().ok()?;
            if let syn::Meta::NameValue(mnv) = meta {
                if let syn::Lit::Str(litstr) = mnv.lit {
                    return Some(litstr.value());
                }
            }
            None
        });
        if purpose.is_none() {
            quote_spanned! {
                name.span() =>
                compile_error!(
                    "cvars should always include doc comments indicating \
                     their purpose",
                );
            };
        }
    }

    // Codegen pass: construct cvar declarations for each entry
    let mut declarations = quote! {};
    for ConfigDeclaration {
        attrs,
        visibility,
        name,
        ty,
        default_value,
    } in configs.iter()
    {
        let purpose = attrs
            .iter()
            .find_map(|a| {
                let meta = a.parse_meta().ok()?;
                if let syn::Meta::NameValue(mnv) = meta {
                    if let syn::Lit::Str(litstr) = mnv.lit {
                        return Some(litstr.value());
                    }
                }
                None
            })
            .unwrap_or_else(|| "undocumented".to_string());
        let name_str = name.to_string();
        let type_str = quote!(#ty).to_string();
        let decl = quote! {
            #(#attrs)*
            #visibility static #name: config::Config<#ty> = config::Config {
                name: #name_str,
                type_str: #type_str,
                path: concat!(module_path!(), "::", stringify!(#name)),
                purpose: #purpose,
                default_value: || -> #ty { #default_value },
                default_value_str: stringify!(#default_value),
                __init: std::sync::Once::new(),
                __value:
                    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()),
                __next: std::cell::Cell::new(None),
            };
            config::config_registrar!(#name);
        };
        declarations = quote! {
            #declarations
            #decl
        };
    }

    declarations.into()
}

/// Create a premain program initialization entrypoint for the given cvar
#[proc_macro]
pub fn config_registrar(input: RustcTokenStream) -> RustcTokenStream {
    let cfg_ident = parse_macro_input!(input as Ident);

    let wasm_inner = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::config::premain_support::export_everywhere]
        pub extern fn [< __premain_config_registrar_ #cfg_ident >]() {
            #cfg_ident.init();
        }
    };
    let native_inner = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::config::premain_support::ctor]
        fn [< __premain_config_registrar_ #cfg_ident >]() {
            #cfg_ident.init();
        }
    };

    let ast = quote! {
        ::config::premain_support::item! {
            #[cfg(target_arch="wasm32")]
            #wasm_inner
            #[cfg(not(target_arch="wasm32"))]
            #native_inner
        }
    };

    ast.into()
}
