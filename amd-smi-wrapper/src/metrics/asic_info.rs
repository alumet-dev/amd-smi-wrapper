use std::mem::MaybeUninit;

use amd_smi_wrapper_sys::{
    load::VersionedLib,
    versions::{v6_3_0, v6_4_0, v7_0_0},
};

use crate::{handles::AmdProcessorHandle, utils::c_buffer_to_string};

/// Higher-level version of `amdsmi_asic_info_t`.
#[derive(Debug, Default, Clone)]
pub struct AmdAsicInfo {
    /// Model name of a GPU.
    pub market_name: String,
    /// GPU identification given by vendor.
    pub vendor_id: u32,
    /// Complete commercial name of a GPU.
    pub vendor_name: String,
    pub subvendor_id: u32,
    /// Software ID of a GPU.
    pub device_id: u64,
    /// Revision ID of a GPU.
    pub rev_id: u32,
    /// GPU chip serial number.
    pub asic_serial: String,
    /// Open Application Model identification.
    pub oam_id: u32,
    /// Total number of compute unit on a GPU.
    pub num_of_compute_units: u32,
    /// Graphic core version of a GPU.
    pub target_graphics_version: u64,
    /// GPU identification given by subsystem.
    /// Only available on some versions of AMD SMI.
    pub subsystem_id: Option<u32>,
}

impl AmdAsicInfo {
    pub fn get(handle: &AmdProcessorHandle) -> Result<Self, crate::error::AmdError> {
        let stable = &handle.amdsmi.shared.inner.lib_stable;
        let info = match &handle.amdsmi.shared.inner.lib_versioned {
            VersionedLib::V6_3_0(_) => {
                // Allocate uninitialized memory for the structure and avoid reading it before the FFI call.
                let mut info = MaybeUninit::<v6_3_0::amdsmi_asic_info_t>::uninit();

                // SAFETY:
                // - According to AMD-SMI documentation, the function fully initializes the structure on success.
                // - The return code is checked before using the data.
                // - Every struct pointer has the same size.
                let res = unsafe {
                    stable.amdsmi_get_gpu_asic_info(handle.inner, info.as_mut_ptr() as _)
                };
                handle.amdsmi.check_status(res)?;

                let info = unsafe { info.assume_init() };
                AmdAsicInfo::from(info)
            }
            VersionedLib::V6_4_0(_) | VersionedLib::V6_4_2(_) => {
                // asic_info_t is the same in 6.4.0 and 6.4.2
                let mut info = MaybeUninit::<v6_4_0::amdsmi_asic_info_t>::uninit();
                let res = unsafe {
                    stable.amdsmi_get_gpu_asic_info(handle.inner, info.as_mut_ptr() as _)
                };
                handle.amdsmi.check_status(res)?;
                let info = unsafe { info.assume_init() };
                AmdAsicInfo::from(info)
            }
            VersionedLib::V7_0_0(_) | VersionedLib::V7_2_0(_) => {
                // asic_info_t is the same in 7.0.0 and 7.2.0
                let mut info = MaybeUninit::<v7_0_0::amdsmi_asic_info_t>::uninit();
                let res = unsafe {
                    stable.amdsmi_get_gpu_asic_info(handle.inner, info.as_mut_ptr() as _)
                };
                handle.amdsmi.check_status(res)?;
                let info = unsafe { info.assume_init() };
                AmdAsicInfo::from(info)
            }
        };
        Ok(info)
    }
}

impl From<v6_3_0::amdsmi_asic_info_t> for AmdAsicInfo {
    fn from(info: v6_3_0::amdsmi_asic_info_t) -> Self {
        Self {
            market_name: c_buffer_to_string(&info.market_name),
            vendor_id: info.vendor_id,
            vendor_name: c_buffer_to_string(&info.vendor_name),
            subvendor_id: info.subvendor_id,
            device_id: info.device_id,
            rev_id: info.rev_id,
            asic_serial: c_buffer_to_string(&info.asic_serial),
            oam_id: info.oam_id,
            num_of_compute_units: info.num_of_compute_units,
            target_graphics_version: info.target_graphics_version,
            subsystem_id: None,
        }
    }
}

impl From<v6_4_0::amdsmi_asic_info_t> for AmdAsicInfo {
    fn from(info: v6_4_0::amdsmi_asic_info_t) -> Self {
        Self {
            market_name: c_buffer_to_string(&info.market_name),
            vendor_id: info.vendor_id,
            vendor_name: c_buffer_to_string(&info.vendor_name),
            subvendor_id: info.subvendor_id,
            device_id: info.device_id,
            rev_id: info.rev_id,
            asic_serial: c_buffer_to_string(&info.asic_serial),
            oam_id: info.oam_id,
            num_of_compute_units: info.num_of_compute_units,
            target_graphics_version: info.target_graphics_version,
            subsystem_id: None,
        }
    }
}

impl From<v7_0_0::amdsmi_asic_info_t> for AmdAsicInfo {
    fn from(info: v7_0_0::amdsmi_asic_info_t) -> Self {
        Self {
            market_name: c_buffer_to_string(&info.market_name),
            vendor_id: info.vendor_id,
            vendor_name: c_buffer_to_string(&info.vendor_name),
            subvendor_id: info.subvendor_id,
            device_id: info.device_id,
            rev_id: info.rev_id,
            asic_serial: c_buffer_to_string(&info.asic_serial),
            oam_id: info.oam_id,
            num_of_compute_units: info.num_of_compute_units,
            target_graphics_version: info.target_graphics_version,
            subsystem_id: Some(info.subsystem_id),
        }
    }
}
