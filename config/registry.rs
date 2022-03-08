//! Global Configuration Registry
//! =============================
//!
//! This module provides a global registry of [`Config
//! containers`](crate::container::Config), stitched together in an intrusive
//! linked list.

use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    /// Global registry of config values
    pub static ref REGISTRY: ConfigRegistry = {
        ConfigRegistry::new()
    };
}

/// An object which can be inserted in the global registry of configs
pub trait Configurable: std::fmt::Debug {
    /// Get the next Configurable in the global registry
    fn get_next(&'static self) -> Option<&dyn Configurable>;
    /// Update which Configurable appears next in the registry
    fn set_next(&'static self, next: Option<&'static dyn Configurable>);

    /// Check if a RON string can be deserialized to the correct type for
    /// this configurable.
    fn typecheck(&'static self, ron: &str) -> anyhow::Result<()>;
    /// Update the value of this configurable using a RON string
    fn set_from_ron(&'static self, ron: &str) -> anyhow::Result<()>;
    /// Get the value of this configurable as a RON string
    fn as_ron(&'static self) -> String;

    /// Get the name of this configurable
    fn get_name(&'static self) -> &'static str;
    /// Get a string representation of the type of this config variable
    fn get_type(&'static self) -> &'static str;
    /// Get the path to this configurable
    fn get_path(&'static self) -> &'static str;
    /// Get the purpose of this configurable
    fn get_purpose(&'static self) -> &'static str;
    /// Get a string representation of how the default value for this
    /// configurable is constructed
    fn get_default_value(&'static self) -> &'static str;
}

/// Iterator type for traversing the global configuration registry
pub struct ConfigIterator {
    current: Option<&'static dyn Configurable>,
}
impl ConfigIterator {
    #[inline]
    /// Create a new config iterator, beginning at the given Configurable
    pub fn new(ptr: &'static dyn Configurable) -> Self {
        ConfigIterator { current: Some(ptr) }
    }
    #[inline]
    /// Create a new empty config iterator -- one that will only yield None
    pub fn empty() -> Self {
        ConfigIterator { current: None }
    }
    #[inline]
    /// Create a new config iterator from an Option — if `opt` is None, so too
    /// the iterator be. If `opt` has a value, the iterator will begin with it
    /// and traverse the list beginning there.
    pub fn from_option(opt: Option<&'static dyn Configurable>) -> Self {
        ConfigIterator { current: opt }
    }
}
impl std::iter::Iterator for ConfigIterator {
    type Item = &'static dyn Configurable;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.get_next();
        Some(current)
    }
}

/// A registry of configs, represented as an intrusive linked list
pub struct ConfigRegistry {
    configs: RwLock<Option<&'static dyn Configurable>>,
}
unsafe impl Send for ConfigRegistry {}
unsafe impl Sync for ConfigRegistry {}
impl ConfigRegistry {
    fn new() -> Self {
        log::trace!("initializing config registry");
        ConfigRegistry {
            configs: RwLock::new(None),
        }
    }
    /// Get an iterator over all globally registered configs
    pub fn iter(&self) -> ConfigIterator {
        let head = *self.configs.read().expect("config registry was poisoned");
        ConfigIterator::from_option(head)
    }
    /// Register a new [`Configurable`] in the global registry
    pub fn register(&self, new_config: &'static dyn Configurable) {
        let mut guard =
            self.configs.write().expect("config registry was poisoned");
        new_config.set_next(*guard);
        *guard = Some(new_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        for cfg in REGISTRY.iter() {
            println!("{:#?}", cfg);
        }
    }
}
