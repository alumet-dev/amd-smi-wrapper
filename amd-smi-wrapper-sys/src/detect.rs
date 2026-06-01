use std::mem::MaybeUninit;

use super::versions;
use crate::versions::stable;
use libloading::Symbol;
use thiserror::Error;

#[derive(Debug)]
pub struct AmdSmiVersion {
    /// Compatible version of ROC-m, for instance [6, 3, 0] for v6.3.0.
    pub rocm_version: [u32; 3],
    /// Version of the AMD SMI library, for instance [24, 7, 1, 0] for v24.7.1.0.
    pub smi_version: [u32; 4],
    /// Is this version in the range that has been tested with these bindings?
    pub is_officially_supported: bool,
}

#[derive(Error, Debug)]
pub enum DetectError {
    #[error("failed to load symbols from the AMD SMI library")]
    Symbols(#[from] libloading::Error),
    #[error("unsupported version of AMD SMI: {} - only versions >= 24.7.1 are supported", format_version(.0))]
    UnsupportedOld([u32; 4]),
    #[error("unsupported version of AMD SMI: {} - only versions <= 26.2.1 are supported", format_version(.0))]
    UnsupportedNew([u32; 4]),
    #[error("failed to get the version of the AMD SMI library: error {}", .0.0)]
    GetVersion(stable::amdsmi_status_t),
}

fn format_version(v: &[u32; 4]) -> String {
    format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3])
}

/// Detects the version of the AMD SMI library that is installed on the system.
pub fn detect_version(
    library: &libloading::Library,
    allow_newer_versions: bool,
) -> Result<AmdSmiVersion, DetectError> {
    // Unfortunately, the version struct is NOT stable in AMD SMI (it changed between v6 and v7).
    // Let's call the newest version and change if it's wrong.
    let version_fn: Symbol<VersionFn> = unsafe { library.get(b"amdsmi_get_lib_version") }?;
    let mut version: MaybeUninit<amdsmi_version_unknown> = MaybeUninit::uninit();
    let result = unsafe { (*version_fn)(version.as_mut_ptr()) };
    if result != stable::AMDSMI_STATUS_SUCCESS {
        return Err(DetectError::GetVersion(result));
    }
    let version = unsafe { version.assume_init() };

    // SAFETY: the first field is the same in both v6 and v7, so it's ok to read either.
    let version_major = unsafe { version.v7.major };
    if version_major >= 26 {
        let full_version_v7 = unsafe { version.v7 };
        let smi_version = [
            full_version_v7.major,
            full_version_v7.minor,
            full_version_v7.release,
            0,
        ];
        let is_officially_supported = smi_version <= [26, 2, 2, 0];

        if !is_officially_supported && !allow_newer_versions {
            return Err(DetectError::UnsupportedNew(smi_version));
        }

        Ok(AmdSmiVersion {
            rocm_version: [7, full_version_v7.minor, 0],
            smi_version,
            is_officially_supported,
        })
    } else {
        let full_version_v6 = unsafe { version.v6 };
        let smi_version = [
            full_version_v6.year,
            full_version_v6.major,
            full_version_v6.minor,
            full_version_v6.release,
        ];

        if smi_version < [24, 7, 1, 0] {
            return Err(DetectError::UnsupportedOld(smi_version));
        }

        let rocm_version = if smi_version < [25, 25, 3, 0] {
            // >= 24.7.1.0 < 25.25.3.0
            [6, 3, 0]
        } else if smi_version < [25, 25, 5, 1] {
            // >= 25.25.3.0 < 25.25.5.1
            [6, 4, 0]
        } else {
            // >= 25.25.5.1 < 26.*
            [6, 4, 2]
        };

        Ok(AmdSmiVersion {
            rocm_version,
            smi_version: [
                full_version_v6.year,
                full_version_v6.major,
                full_version_v6.minor,
                full_version_v6.release,
            ],
            is_officially_supported: true,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
union amdsmi_version_unknown {
    pub v6: versions::v6_3_0::amdsmi_version_t,
    pub v7: versions::v7_0_0::amdsmi_version_t,
}

type VersionFn =
    unsafe extern "C" fn(version: *mut amdsmi_version_unknown) -> versions::stable::amdsmi_status_t;
