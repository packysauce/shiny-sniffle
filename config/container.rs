//! Static Configuration Containers
//! ===============================
//!
//! This module declares the container type which wraps configuration variables.
//! At this layer they're not inherently global yet, they're just metadata,
//! value storage, default initializers, and some intrusive linked list pointers
//! (which aren't default-wired-up to anything).
//!
//! If, however, you call `init` on one of these, it'll add itself to the global
//! registry's linked list of configuration variables.

use crate::registry::{Configurable, REGISTRY};
use std::cell::Cell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Once, RwLock};

/// Configs are lightweight accessors for global configuration variables.
///
/// These only expose a very simple get/set interface for using the underlying
/// value from the global store, and are meant to be declared statically using
/// the `config!` macro.
pub struct Config<T>
where
    T: 'static + std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    /// The name of this configuration variable
    pub name: &'static str,
    /// The type of this configuration variable, formatted to a string
    pub type_str: &'static str,
    /// The module path at which this configuration variable can be found
    ///
    /// This is a `::` delimited string which you could copy in to rust source
    /// and it'd compile, e.g. `somelib::somemod::A_CONFIG_VAR`.
    pub path: &'static str,
    /// A human-readable description of what this configuration variable is
    /// meant to be used for.
    pub purpose: &'static str,
    /// A human-readable representation of the default value initializer of
    /// this configuration variable.
    pub default_value_str: &'static str,
    /// Initializer used to fill this cvar with a default value if none is
    /// explicitly set before the first read.
    pub default_value: fn() -> T,

    // These are marked public to work around a `const fn` deficiency on
    // generic types. You probably don't want to access them.
    // See https://github.com/rust-lang/rust/issues/57563
    /// PRIVATE FIELD
    pub __init: Once,
    /// PRIVATE FIELD
    pub __value: AtomicPtr<RwLock<T>>,
    /// PRIVATE FIELD
    pub __next: Cell<Option<&'static dyn Configurable>>,
}

// Mark Config objects safe to pass between threads
unsafe impl<T> Send for Config<T> where
    T: std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>
{
}
// Mark Config objects safe to share between threads
unsafe impl<T> Sync for Config<T> where
    T: std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>
{
}

impl<T> std::fmt::Debug for Config<T>
where
    T: 'static + std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config<{}> @ {}", std::any::type_name::<T>(), self.path)
    }
}

impl<T> Configurable for Config<T>
where
    T: 'static + std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    fn get_next(&'static self) -> Option<&dyn Configurable> {
        self.__next.get()
    }
    fn set_next(&'static self, next: Option<&'static dyn Configurable>) {
        self.__next.replace(next);
    }

    fn typecheck(&'static self, ron: &str) -> anyhow::Result<()> {
        ron::de::from_str::<T>(ron)?;
        Ok(())
    }
    fn set_from_ron(&'static self, ron: &str) -> anyhow::Result<()> {
        let new_value = ron::de::from_str::<T>(ron)?;
        self.set(new_value);
        Ok(())
    }
    fn as_ron(&'static self) -> String {
        ron::ser::to_string(&self.get()).expect("Serializing config failed")
    }

    fn get_name(&'static self) -> &'static str {
        self.name
    }
    fn get_type(&'static self) -> &'static str {
        self.type_str
    }
    fn get_path(&'static self) -> &'static str {
        self.path
    }
    fn get_purpose(&'static self) -> &'static str {
        self.purpose
    }
    fn get_default_value(&'static self) -> &'static str {
        self.default_value_str
    }
}

impl<T> Config<T>
where
    T: 'static + std::fmt::Debug + Clone + serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    #[inline]
    /// Initialize this configuration variable, stitching it up to the global
    /// registry and setting it to its default value
    pub fn init(&'static self) {
        self.__init.call_once(|| {
            let lock = RwLock::new((self.default_value)());
            let lockbox = Box::new(lock);
            self.__value.store(Box::leak(lockbox), Ordering::SeqCst);
            REGISTRY.register(self);
        });
    }

    #[inline]
    /// Iterate over all previously-registered configuration variables
    pub fn iter() -> impl Iterator<Item = &'static dyn Configurable> {
        REGISTRY.iter()
    }

    /// Get the value of this configuration variable.
    pub fn get(&'static self) -> T {
        let vl_ptr = self.__value.load(Ordering::SeqCst);
        let value_lock = unsafe { &*vl_ptr };
        let guard = value_lock.read().expect("configuration guard poisoned");
        (*guard).clone()
    }

    /// Set the value of this configuration variable
    pub fn set(&'static self, value: T) {
        let vl_ptr = self.__value.load(Ordering::SeqCst);
        let value_lock = unsafe { &*vl_ptr };
        let mut guard = value_lock.write().expect("configuration guard poisoned");
        *guard = value;
    }
}
