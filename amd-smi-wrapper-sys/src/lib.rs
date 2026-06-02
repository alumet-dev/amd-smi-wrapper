//! Low-level bindings to AMD SMI.
//!
//! # Example
//!
//! ```no_run
//! use amd_smi_wrapper_sys::{load_and_init, versions::stable::amdsmi_init_flags_t};
//! let flags = amdsmi_init_flags_t::AMDSMI_INIT_AMD_GPUS;
//! let lib = load_and_init("libamd_smi.so", true, flags).expect("load failure");
//! println!("AMD SMI version: {:?}", lib.version);
//! ```
//!
//! For more details, see the integration tests of this package.
//!
//! # Multi-version Support
//!
//! Unfortunately, each version of AMD SMI can introduce breaking changes, and
//! [has in the past](https://github.com/alumet-dev/amd-smi-wrapper/issues/3), even between minor versions!
//!
//! This crate **automatically detects** the version of AMD SMI.
//! After loading the library, you will have access to:
//! - a `stable` subset of AMD SMI, with the functions and structs that don't change between versions
//! - a `versioned` subset of AMD SMI for a given version, with the functions and structs that are unstable

/// Bindings to multiple versions of amdsmi.
#[allow(warnings)]
pub mod versions;

/// Detection of the system version of amdsmi.
pub mod detect;

/// Loading of amdsmi.
pub mod load;

pub use load::load_and_init;
