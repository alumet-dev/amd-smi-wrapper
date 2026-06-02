//! High-level wrapper for AMD SMI.
//!
//! The AMD SMI library is _dynamically_ loaded.
//! You don't need to have AMD SMI installed to compile this crate.
//!
//! # Example
//! ```no_run
//! use amd_smi_wrapper::{AmdSmi, AmdInitFlags, AmdInterface};
//! use amd_smi_wrapper::handles::{ProcessorHandle, SocketHandle};
//!
//! let amdsmi = AmdSmi::init(AmdInitFlags::AMDSMI_INIT_AMD_GPUS)?;
//!
//! let version = amdsmi.version().smi_version;
//! println!("loaded AMD SMI version {}.{}.{}.{}", version[0], version[1], version[2], version[3]);
//!
//! for socket in amdsmi.socket_handles()? {
//!     for proc in socket.processor_handles()? {
//!         let uuid = proc.device_uuid()?;
//!         println!("Detected GPU: {uuid}");
//!         let power_info = proc.device_power_info()?;
//!         match power_info.current_socket_power {
//!             Some(power) => {
//!                 println!("Current Power (W): {power}");
//!             }
//!             None => {
//!                 // not all GPUs support power metrics
//!                 println!("Current Power (W): unsupported");
//!             }
//!         }
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Multi-version Support
//!
//! Unfortunately, each version of AMD SMI can introduce breaking changes, and
//! [has in the past](https://github.com/alumet-dev/amd-smi-wrapper/issues/3), even between minor versions!
//!
//! This crate **automatically detects** the version of AMD SMI and adapts to the available features.
//! Some metrics are only available with the most recent versions of AMD SMI and/or the most recent GPUs.
//! That is why some fields in metric structures are `Option`s.
//! One example of that is [`metrics::power_info::AmdPowerInfo::current_socket_power`].
//!
//! You can check [`AmdSmiVersion::is_officially_supported`] to see whether the version of AMD SMI that is installed on the system has been tested with this crate.
//! If this flag is `false`, it may or may not work, depending on the compatibility efforts of AMD.
//!
//! Currently supported versions: ROCm **v6.3.0 - v7.2.x**
//!
//! # Mocking Support
//!
//! With `amd-smi-wrapper`, you can easily create mock structure for your tests.
//! To do so, enable the `mock` feature and use [`MockAmdInterface`].
#![deny(unsafe_op_in_unsafe_fn)]
use std::{ptr::null_mut, sync::Arc};

#[cfg(feature = "mock")]
use mockall::automock;

pub mod error;
pub mod handles;
pub mod metrics;
mod utils;

use crate::{
    error::{AmdError, AmdInitError},
    handles::{AmdSocketHandle, SocketHandle},
};
use amd_smi_wrapper_sys::detect::AmdSmiVersion;
use amd_smi_wrapper_sys::{load::MultiVersionLib, versions::stable};

pub(crate) const LIB_PATH: &str = "libamd_smi.so";

/// Initialization flags for the library.
/// See [`AmdSmi::init`].
pub type AmdInitFlags = stable::amdsmi_init_flags_t;

struct LibAmdSmi {
    inner: MultiVersionLib,
}

/// Main wrapper around the AMD SMI library.
///
/// # Shutdown
/// The library is automatically shut down when `AmdSmi` is dropped.
/// The `Drop` implementation of `AmdSmi` ignores shutdown errors.
#[derive(Clone)]
pub struct AmdSmi {
    shared: Arc<LibAmdSmi>,
}

impl Drop for LibAmdSmi {
    fn drop(&mut self) {
        // Shut down the AMD-SMI library and release all internal resources.
        // SAFETY: The function expects a valid, initialized library instance.
        // The shutdown is called only once when the last reference is dropped.
        unsafe { self.inner.lib_stable.amdsmi_shut_down() };
    }
}

impl AmdSmi {
    /// Checking the value of [`amdsmi_status_t`] to return an error or success.
    fn check_status(&self, status: stable::amdsmi_status_t) -> Result<(), AmdError> {
        match status {
            stable::AMDSMI_STATUS_SUCCESS => Ok(()),
            other => Err(self.build_error(other)),
        }
    }

    fn build_error(&self, status: stable::amdsmi_status_t) -> AmdError {
        assert_ne!(status, stable::AMDSMI_STATUS_SUCCESS);
        AmdError::from_status_with_message(status, &self)
    }

    /// Initializes the AMD smi library.
    ///
    /// # Example
    /// ```no_run
    /// use amd_smi_wrapper::{AmdSmi, AmdInitFlags};
    ///
    /// let amdsmi = AmdSmi::init(AmdInitFlags::AMDSMI_INIT_AMD_GPUS).expect("init failed");
    /// ```
    pub fn init(flags: AmdInitFlags) -> Result<Self, AmdInitError> {
        log::debug!("Initializing AMD SMI...");
        let amdsmi = amd_smi_wrapper_sys::load_and_init(LIB_PATH, true, flags)?;
        log::debug!("AMD SMI initialized. Version info: {:?}", amdsmi.version);

        let instance = AmdSmi {
            shared: Arc::new(LibAmdSmi { inner: amdsmi }),
        };
        Ok(instance)
    }

    /// Version info about the AMD SMI library that has been loaded.
    pub fn version(&self) -> &AmdSmiVersion {
        &self.shared.inner.version
    }
}

/// Provides AMD SMI functions.
///
/// The actual implementation is [`AmdSmi`].
/// In tests, you can use the mock implementation `MockAmdInterface` (requires the `mock` feature).
#[cfg_attr(feature = "mock", automock(type SocketHandle=handles::MockSocketHandle;))]
pub trait AmdInterface {
    /// Type of socket handle managed by this interface.
    type SocketHandle: SocketHandle;

    /// Lists the available sockets.
    ///
    /// Only the sockets that match the initialization flags are returned.
    /// For instance, if the library has been initialized with [`AMDSMI_INIT_AMD_GPUS`](AmdInitFlags::AMDSMI_INIT_AMD_GPUS),
    /// only sockets with GPUs are returned.
    fn socket_handles(&self) -> Result<Vec<Self::SocketHandle>, AmdError>;
}

impl AmdInterface for AmdSmi {
    type SocketHandle = AmdSocketHandle;

    fn socket_handles(&self) -> Result<Vec<Self::SocketHandle>, AmdError> {
        let mut socket_count = 0;

        // Query the number of available GPU socket handles.
        // SAFETY: According to the AMD-SMI documentation, passing `null_mut()` is safe which sets `socket_count` to the number of sockets in the system.
        let result = unsafe {
            self.shared
                .inner
                .lib_stable
                .amdsmi_get_socket_handles(&mut socket_count, null_mut())
        };
        self.check_status(result)?;

        // Allocate a vector of null pointers.
        let mut socket_handles = vec![null_mut(); socket_count as usize];

        // Fill the buffer with socket handles.
        // SAFETY: `socket_handles.as_mut_ptr()` points to memory of sufficient size.
        // According to the AMD-SMI library documentation, the function writes at most `socket_count` handles, so no out-of-bounds write occurs.
        let result = unsafe {
            self.shared
                .inner
                .lib_stable
                .amdsmi_get_socket_handles(&mut socket_count, socket_handles.as_mut_ptr())
        };
        self.check_status(result)?;

        socket_handles.truncate(socket_count as usize);

        Ok(socket_handles
            .into_iter()
            .map(|s| AmdSocketHandle {
                amdsmi: self.clone(),
                inner: s,
            })
            .collect())
    }
}
