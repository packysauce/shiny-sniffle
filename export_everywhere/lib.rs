//! This crate defines the `#[export_everywhere]` attribute for functions that
//! should be exported natively in cdylibs or via `wasm_bindgen` if that target
//! is selected.

// The proc-macro compiler dependency is injected in a lib segment of Cargo.toml
// right now, rather than as a dependency, so the compiler will fail to resolve
// it without an ed2015-style `extern crate` statement. This is intended to be
// a temporary situation, but it's still where we are today (August 2019).
// ref: https://users.rust-lang.org/t/how-to-use-proc-macro-on-rust-2018/20833/5
#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream as RustcTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        fn gen_export_attr() -> TokenStream {
            quote! {
                #[wasm_bindgen]
            }
        }
        fn gen_entrypoint_attr() -> TokenStream {
            quote! {
                #[wasm_bindgen]
            }
        }
    } else {
        fn gen_export_attr() -> TokenStream {
            quote! {
                #[no_mangle]
            }
        }
        fn gen_entrypoint_attr() -> TokenStream {
            // entrypoint replacement with #[start] is an unstable feature still
            // so for now we _have_ to name the function `main`
            // https://github.com/rust-lang/rust/issues/29633
            // quote! {
            //     #[start]
            // }
            quote! {}
        }
    }
}

/// Mark an entrypoint function, either to use as a native binary or from a
/// wasm loader
#[proc_macro_attribute]
pub fn export_main(_attr: RustcTokenStream, input: RustcTokenStream) -> RustcTokenStream {
    let input_ast = parse_macro_input!(input as ItemFn);

    let fn_sig_span: Span = input_ast.sig.ident.span();

    // Assert that the target function is named `main`
    if input_ast.sig.ident != "main" {
        let msg = format!(
            "fn must be named `main` to mark as an entrypoint (not {})",
            &input_ast.sig.ident
        );
        let err = syn::Error::new(input_ast.sig.ident.span(), msg);
        return err.to_compile_error().into();
    }

    // Assert that the target function is `pub`
    #[allow(clippy::single_match_else)]
    match input_ast.vis {
        syn::Visibility::Public(_) => {}
        _ => {
            let msg = format!(
                "fn `{}` must be `pub` to mark as an entrypoint",
                &input_ast.sig.ident
            );
            let err = syn::Error::new(fn_sig_span, msg);
            return err.to_compile_error().into();
        }
    }

    let entrypoint_attr = gen_entrypoint_attr();
    let expanded = quote! {
        #entrypoint_attr
        #input_ast
    };

    expanded.into()
}

/// Export a function, either as a no-mangle extern, or via wasm-bindgen,
/// depending on the target platform.
#[proc_macro_attribute]
pub fn export_everywhere(_attr: RustcTokenStream, input: RustcTokenStream) -> RustcTokenStream {
    let input_ast = parse_macro_input!(input as ItemFn);

    let fn_sig_span: Span = input_ast.sig.ident.span();

    // Assert that the target function is `extern`
    if input_ast.sig.abi.is_none() {
        let msg = format!(
            "fn `{}` must be marked `extern` to export_everywhere",
            &input_ast.sig.ident
        );
        let err = syn::Error::new(input_ast.sig.ident.span(), msg);
        return err.to_compile_error().into();
    }

    // Assert that the target function is `pub`
    #[allow(clippy::single_match_else)]
    match input_ast.vis {
        syn::Visibility::Public(_) => {}
        _ => {
            let msg = format!(
                "fn `{}` must be marked `pub` to export_everywhere",
                &input_ast.sig.ident
            );
            let err = syn::Error::new(fn_sig_span, msg);
            return err.to_compile_error().into();
        }
    }

    let export_attr = gen_export_attr();
    let expanded = quote! {
        #export_attr
        #input_ast
    };

    expanded.into()
}
