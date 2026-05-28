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
/// To handle the error, call [`AmdInterface::stop`].
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
        let amdsmi = amd_smi_wrapper_sys::load(LIB_PATH, true)?;
        log::debug!("Loaded AMD SMI library. Version info: {:?}", amdsmi.version);
        let instance = AmdSmi {
            shared: Arc::new(LibAmdSmi { inner: amdsmi }),
        };

        log::debug!("Calling amdsmi_init({flags:?})...");
        // SAFETY: The function expects a valid library instance and valid flags.
        // According to the AMD-SMI documentation, the function fully initializes internal structures for GPU discovery.
        // The return code `amdsmi_status_t` is checked to ensure initialization succeeded before using the library.
        let status = unsafe { instance.shared.inner.lib_stable.amdsmi_init(flags.0.into()) };
        instance.check_status(status)?;
        log::debug!("AMD SMI is ready.");

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
