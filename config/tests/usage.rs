use config::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct CustomType {
    value: usize,
}

config! {
    /// Testing string-shaped configs
    STRINGY: String = "Hello, world!".to_string();
    /// A cvar just for clobbering with the set test
    CLOBBERME: String = "Hello, world!".to_string();
    /// A more complicated initializer for a string
    COMPLICATED_STRING: String = {
        let first = "Hello";
        let second = "world";
        format!("{}, {}!", first, second)
    };
}

#[test]
fn string_config() {
    assert_eq!(STRINGY.get(), "Hello, world!");
}
#[test]
fn complicated_string_config() {
    assert_eq!(COMPLICATED_STRING.get(), "Hello, world!");
}
#[cfg(not(all(target_arch = "wasm32")))]
#[test]
fn registration() {
    let mut stringy_found = false;
    let mut cs_found = false;
    for cfg in REGISTRY.iter() {
        if cfg.get_name() == "STRINGY" {
            stringy_found = true;
        }
        if cfg.get_name() == "COMPLICATED_STRING" {
            cs_found = true;
        }
    }
    assert!(stringy_found, "Found the test cvar STRINGY");
    assert!(cs_found, "Found the test cvar COMPLICATED_STRING");
}
#[test]
fn metadata() {
    let cfg = &STRINGY as &dyn Configurable;
    assert_eq!(cfg.get_name(), "STRINGY");
    assert_eq!(cfg.get_type(), "String");
    assert_eq!(cfg.get_path(), concat!(module_path!(), "::STRINGY"));
    assert_eq!(cfg.get_purpose(), " Testing string-shaped configs");
    assert_eq!(cfg.get_default_value(), r#""Hello, world!" . to_string()"#);
}
#[test]
fn typechecking() {
    let cfg = &STRINGY as &dyn Configurable;
    assert!(cfg.typecheck("5").is_err(), "Number");
    assert!(cfg.typecheck("[]").is_err(), "Array");
    assert!(cfg.typecheck("{}").is_err(), "Object");
    assert!(cfg.typecheck("test").is_err(), "Unquoted string");
    assert!(cfg.typecheck(r#""test""#).is_ok(), "Double quoted string");
}
#[cfg(not(all(target_arch = "wasm32")))]
#[test]
fn set_from_registry() {
    let cfg = find("CLOBBERME").pop().unwrap();
    assert_eq!(cfg.get_name(), CLOBBERME.get_name());
    let new_value = r#""Hi, I am a RON string!""#;
    cfg.set_from_ron(new_value).expect("serializing failed!");
    assert_eq!(cfg.as_ron(), new_value);
}
#[cfg(not(all(target_arch = "wasm32")))]
#[test]
fn searching() {
    assert_eq!(find("STRINGY").len(), 1);

    let search_path = concat!(module_path!(), "::", "STRINGY");
    assert_eq!(lookup(search_path).unwrap().get_name(), STRINGY.get_name());
}
