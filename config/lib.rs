//! Configs are global configuration variables.
//!
//!
//! These work very similarly to the global configuration systems we've had in
//! the past -- in particular they're a log like nitrogen cvars, but with
//! a good deal more flexibility. We maintain a global intrusive linked list
//! of all config variables which must be registered before any are used. We
//! do this automatically premain on platforms where the `ctor` crate is
//! supported, and manually in the JS loader on wasm.
//!
//! Note that docstrings for each config element are required, and recorded on
//! the config as the "purpose" field, so be nice to future-you and add useful
//! stuff there!
//!
//! # Examples
//!
//! You can use any serializable primitive type in a config variable:
//! ```
//! # mod config_demo {
//! use config::config;
//! config! {
//!     /// Width of the game window
//!     RES_X: i16 = 640;
//!     /// Height of the game window
//!     RES_Y: i16 = 480;
//! }
//! fn open_window() {
//!     println!("opening a {}x{} window", RES_X.get(), RES_Y.get());
//! }
//! # }
//! ```
//!
//! That includes compound types like tuples (and structs), too:
//! ```
//! # mod config_demo {
//! use config::config;
//! config! {
//!     /// Resolution of the game window
//!     RESOLUTION: (i16, i16) = (640, 480);
//! }
//! fn open_window() {
//!     let (rx, ry) = RESOLUTION.get();
//!     println!("opening a {}x{} window", rx, ry);
//! }
//! # }
//! ```

pub mod container;
pub mod registry;

pub use crate::container::Config;
pub use crate::registry::{Configurable, REGISTRY};

#[cfg(not(all(target_arch = "wasm32")))]
/// Premain initializer support for non-web targets
pub mod premain_support {
    pub use ::ctor::ctor;
    pub use ::paste::item;
}
#[cfg(all(target_arch = "wasm32"))]
/// Premain initializer support for web targets
pub mod premain_support {
    pub use ::export_everywhere::export_everywhere;
    pub use ::paste::item;
}

/// Define a group of configuration variables for this module.
///
/// Once defined, configuration variables are available as global statics
/// that you can read and update with `get()` / `set()` respectively.
///
/// [`Config`]: struct.Config.html
/// [`config::global`]: global/index.html
/// [`config::tree`]: tree/index.html
pub use config_macros::config;
/// Create a premain registrar function for initializing the given cvar
// NB. This hardcodes the paths to the premain_support reexports. This is
//     brittle, I know, but it's what works right now.
pub use config_macros::config_registrar;

/// Get the list of all configs linked in to this program.
///
/// Any cvar defined in any library linked in to this program is eligible -- if
/// it's part of the binary, it'll appear here.
pub fn all_configs(
) -> impl std::iter::Iterator<Item = &'static dyn registry::Configurable> {
    REGISTRY.iter()
}
/// Find all config variables with the given name
pub fn find<S: AsRef<str>>(name: S) -> Vec<&'static dyn Configurable> {
    let name = name.as_ref();
    all_configs().filter(|cfg| cfg.get_name() == name).collect()
}
/// Look up a specific config variable by module path.
///
/// You can find paths either by locating the symbol in source code, or
/// inspecting the `--help` output from a program which includes that cvar.
/// Paths are strings like `surface::sys::sdl::GL_VERSION`.
pub fn lookup<S: AsRef<str>>(path: S) -> Option<&'static dyn Configurable> {
    let path = path.as_ref();
    all_configs().find(|cfg| cfg.get_path() == path)
}
