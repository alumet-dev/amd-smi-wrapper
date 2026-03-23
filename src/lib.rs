use std::sync::Arc;

mod bindings;
pub mod error;
pub mod handles;
pub mod metrics;
mod utils;

use crate::{
    bindings::{amdsmi_init_flags_t, amdsmi_status_t, libamd_smi},
    error::{AmdStatus, message_status}, handles::SocketHandle,
};
use error::{AmdError, AmdInitError};

pub(crate) const LIB_PATH: &str = "libamd_smi.so";

/// Initialization flags for the library.
/// See [`AmdSmi::init`].
pub type AmdInitFlags = amdsmi_init_flags_t;

/// Main wrapper around the AMD SMI library.
///
/// # Shutdown
/// The library is automatically shut down when `AmdSmi` is dropped.
/// The `Drop` implementation of `AmdSmi` ignores shutdown errors.
/// To handle the error, call [`AmdInterface::stop`].
struct LibAmdSmi {
    amdsmi: libamd_smi,
}

pub struct AmdSmi {
    inner: Arc<LibAmdSmi>,
}

impl Drop for LibAmdSmi {
    fn drop(&mut self) {
        // Shut down the AMD-SMI library and release all internal resources.
        // SAFETY: The function expects a valid, initialized library instance.
        // The shutdown is called only once when the last reference is dropped.
        unsafe { self.amdsmi.amdsmi_shut_down() };
    }
}

impl AmdSmi {
    /// Initializes the AMD smi library.
    ///
    /// # Example
    /// ```no_run
    /// use amd_smi_wrapper::{AmdSmi, AmdInitFlags};
    ///
    /// let amdsmi = AmdSmi::init(AmdInitFlags::AMDSMI_INIT_AMD_GPUS).expect("init failed");
    /// ```
    pub fn init(flags: AmdInitFlags) -> Result<Arc<Self>, AmdInitError> {
        // SAFETY: The library must exist at the specified path, otherwise `libamd_smi::new` returns an error.
        // This operation involves raw FFI interaction and assumes the dynamic loader succeeds.
        let amdsmi = unsafe { libamd_smi::new(LIB_PATH)? };
        let lib = LibAmdSmi { amdsmi };
        let instance = Arc::new(AmdSmi {
            inner: Arc::new(lib),
        });

        // SAFETY: The function expects a valid library instance and valid flags.
        // According to the AMD-SMI documentation, the function fully initializes internal structures for GPU discovery.
        // The return code `amdsmi_status_t` is checked to ensure initialization succeeded before using the library.
        let status = unsafe { instance.inner.amdsmi.amdsmi_init(flags.0.into()) };
        instance.inner.check_status(status)?;

        Ok(instance)
    }
}

impl LibAmdSmi {
    /// Checking the value of [`amdsmi_status_t`] to return an error or success.
    fn check_status(&self, status: amdsmi_status_t) -> Result<(), AmdError> {
        match status {
            AmdStatus::AMDSMI_STATUS_SUCCESS => Ok(()),
            status => Err(AmdError {
                status,
                message: message_status(&self.amdsmi, status),
            }),
        }
    }
}

/// Provides AMD SMI functions.
///
/// The actual implementation is [`AmdSmi`].
/// In tests, you can use the mock implementation `MockAmdInterface` (requires the `mock` feature).
#[cfg_attr(feature = "mock", automock(type SocketHandle=MockSocketHandle;))]
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
