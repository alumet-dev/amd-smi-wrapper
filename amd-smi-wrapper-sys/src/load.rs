use std::{ffi::OsStr, path::PathBuf};

use libloading::Library;
use thiserror::Error;

use crate::{
    detect::{AmdSmiVersion, DetectError},
    versions::{stable, v6_3_0, v6_4_0, v6_4_2, v7_0_0, v7_2_0},
};

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("failed to load the AMD SMI library at {1:?}")]
    Library(#[source] libloading::Error, PathBuf),
    #[error("failed to detect the version of the AMD SMI library")]
    Detect(#[from] DetectError),
}

pub struct MultiVersionLib {
    pub lib_stable: stable::libamd_smi,
    pub lib_versioned: VersionedLib,
    pub version: AmdSmiVersion,
}

pub enum VersionedLib {
    V6_3_0(v6_3_0::libamd_smi),
    V6_4_0(v6_4_0::libamd_smi),
    V6_4_2(v6_4_2::libamd_smi),
    V7_0_0(v7_0_0::libamd_smi),
    V7_2_0(v7_2_0::libamd_smi),
}

/// Load the system version of AMD SMI, detect its version and load version-specific functions.
pub fn load(
    path: impl AsRef<OsStr>,
    allow_newer_versions: bool,
) -> Result<MultiVersionLib, LoadError> {
    // FIXME: technically, we could do one single library load, and then use the right function definitions, but it's not supported by bindgen at the moment (it cannot generate just the definitions, it generates the struct with its version-specific methods, and the same library cannot be shared between multiple structs).
    let path = path.as_ref();

    // Load with the definitions of the stable ABI first.
    // SAFETY: this should be fine, but we cannot guarantee at 100% that AMD SMI won't do something weird.
    // We have no choice here.
    let lib_stable =
        unsafe { stable::libamd_smi::new(path) }.map_err(|e| LoadError::Library(e, path.into()))?;

    // detect the right version and load version-specific methods
    let rawlib = unsafe { Library::new(path) }.map_err(|e| LoadError::Library(e, path.into()))?;
    let version = super::detect::detect_version(&rawlib, allow_newer_versions)?;
    let lib_versioned =
        load_specific_version(rawlib, &version).map_err(|e| LoadError::Library(e, path.into()))?;

    Ok(MultiVersionLib {
        lib_stable,
        lib_versioned,
        version,
    })
}

fn load_specific_version(
    rawlib: Library,
    version: &AmdSmiVersion,
) -> Result<VersionedLib, libloading::Error> {
    let lib_versioned = if version.rocm_version >= [7, 2, 0] {
        VersionedLib::V7_2_0(unsafe { v7_2_0::libamd_smi::from_library(rawlib) }?)
    } else if version.rocm_version >= [7, 0, 0] {
        VersionedLib::V7_0_0(unsafe { v7_0_0::libamd_smi::from_library(rawlib) }?)
    } else if version.rocm_version >= [6, 4, 2] {
        VersionedLib::V6_4_2(unsafe { v6_4_2::libamd_smi::from_library(rawlib) }?)
    } else if version.rocm_version >= [6, 4, 0] {
        VersionedLib::V6_4_0(unsafe { v6_4_0::libamd_smi::from_library(rawlib) }?)
    } else {
        VersionedLib::V6_3_0(unsafe { v6_3_0::libamd_smi::from_library(rawlib) }?)
    };
    Ok(lib_versioned)
}
