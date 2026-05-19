//! Error handling.

use std::{
    ffi::{CStr, c_char},
    fmt::{Display, Formatter},
    ptr,
};
use thiserror::Error;

use super::LIB_PATH;
use amd_smi_wrapper_sys::{load::LoadError, versions::stable};

/// Error while using the AMD SMI library.
#[derive(Error, Debug)]
pub struct AmdError {
    /// The underlying status provided by amdsmi library.
    pub status: stable::amdsmi_status_t,
    /// Detailed description of the error.
    pub message: Option<String>,
}

impl Display for AmdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "amd-smi error {:?}: {msg}", self.status),
            None => write!(f, "amd-smi error {:?}", self.status),
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
