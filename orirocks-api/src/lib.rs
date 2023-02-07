pub mod ffi;
pub mod loader;
pub mod error;
pub mod provider;
mod marshal;

pub use error::Error;

pub const PLUGIN_VERSION: u32 = 1;