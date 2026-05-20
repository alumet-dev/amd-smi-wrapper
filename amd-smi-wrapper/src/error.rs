//! Error handling.

use std::{
    ffi::{CStr, c_char},
    fmt::{Display, Formatter},
    ptr,
};
use thiserror::Error;

use super::{AmdSmi, LIB_PATH};
use amd_smi_wrapper_sys::{load::LoadError, versions::stable};

/// Error while using the AMD SMI library.
#[derive(Error, Debug)]
pub struct AmdError {
    /// The simplified status as a high-level enum.
    pub status: Option<SimplifiedStatus>,
    /// The underlying status code returned by AMD SMI.
    pub status_code: stable::amdsmi_status_t,
    /// Detailed description of the error.
    pub message: Option<String>,
}

impl AmdError {
    /// Creates a new error from a status code, without a description.
    pub fn from_status_code(status_code: stable::amdsmi_status_t) -> Self {
        Self {
            status: SimplifiedStatus::try_from(status_code).ok(),
            status_code,
            message: None,
        }
    }

    /// Creates a new error from a status code, and tries to find a meaningful error message for it.
    pub fn from_status_with_message(status_code: stable::amdsmi_status_t, lib: &AmdSmi) -> Self {
        AmdError {
            status: SimplifiedStatus::try_from(status_code).ok(),
            status_code,
            message: status_message(&lib.shared.inner.lib_stable, status_code),
        }
    }
}

impl Display for AmdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "amd-smi error {}: {msg}", self.status_code.0),
            None => match self.status {
                Some(s) => write!(f, "amd-smi error {}: {s:?}", self.status_code.0),
                None => write!(f, "amd-smi error {}", self.status_code.0),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum AmdInitError {
    #[error("amd-smi init error")]
    Init(#[from] AmdError),
    #[error("Failed to load {LIB_PATH}")]
    Load(#[from] LoadError),
}

/// Returns a detailed description of a status code.
pub fn status_message(
    amdsmi: &stable::libamd_smi,
    status: stable::amdsmi_status_t,
) -> Option<String> {
    let mut status_string: *const c_char = ptr::null();
    let result = unsafe { amdsmi.amdsmi_status_code_to_string(status, &mut status_string) };
    if result == stable::AMDSMI_STATUS_SUCCESS && !status_string.is_null() {
        // SAFETY: the string is null-terminated and the pointer is non-null
        let status_string = unsafe { CStr::from_ptr(status_string) };
        status_string.to_str().ok().map(str::to_string)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimplifiedStatus {
    Success,
    Invalid,
    NotSupported,
    NotYetImplemented,
    FailLoadModule,
    FailLoadSymbol,
    DrmError,
    ApiFailed,
    Timeout,
    Retry,
    NoPermission,
    Interrupt,
    Io,
    AddressFault,
    FileError,
    OutOfResources,
}

impl TryFrom<stable::amdsmi_status_t> for SimplifiedStatus {
    type Error = ();

    fn try_from(status: stable::amdsmi_status_t) -> Result<Self, Self::Error> {
        match status {
            stable::AMDSMI_STATUS_SUCCESS => Ok(Self::Success),
            stable::AMDSMI_STATUS_INVAL => Ok(Self::Invalid),
            stable::AMDSMI_STATUS_NOT_SUPPORTED => Ok(Self::NotSupported),
            stable::AMDSMI_STATUS_NOT_YET_IMPLEMENTED => Ok(Self::NotYetImplemented),
            stable::AMDSMI_STATUS_FAIL_LOAD_MODULE => Ok(Self::FailLoadModule),
            stable::AMDSMI_STATUS_FAIL_LOAD_SYMBOL => Ok(Self::FailLoadSymbol),
            stable::AMDSMI_STATUS_DRM_ERROR => Ok(Self::DrmError),
            stable::AMDSMI_STATUS_API_FAILED => Ok(Self::ApiFailed),
            stable::AMDSMI_STATUS_TIMEOUT => Ok(Self::Timeout),
            stable::AMDSMI_STATUS_RETRY => Ok(Self::Retry),
            stable::AMDSMI_STATUS_NO_PERM => Ok(Self::NoPermission),
            stable::AMDSMI_STATUS_INTERRUPT => Ok(Self::Interrupt),
            stable::AMDSMI_STATUS_IO => Ok(Self::Io),
            stable::AMDSMI_STATUS_ADDRESS_FAULT => Ok(Self::AddressFault),
            stable::AMDSMI_STATUS_FILE_ERROR => Ok(Self::FileError),
            stable::AMDSMI_STATUS_OUT_OF_RESOURCES => Ok(Self::OutOfResources),
            _ => Err(()),
        }
    }
}
