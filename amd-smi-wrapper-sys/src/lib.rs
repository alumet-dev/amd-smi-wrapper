//! Low-level bindings to amdsmi.

/// Bindings to multiple versions of amdsmi.
#[allow(warnings)]
pub mod versions;

/// Detection of the system version of amdsmi.
pub mod detect;

/// Loading of amdsmi.
pub mod load;

pub use load::load_and_init;
