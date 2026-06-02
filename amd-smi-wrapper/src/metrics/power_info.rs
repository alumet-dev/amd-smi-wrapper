use std::mem::MaybeUninit;

use amd_smi_wrapper_sys::{
    load::VersionedLib,
    versions::{v6_3_0, v6_4_0, v7_0_0},
};

use crate::handles::AmdProcessorHandle;

/// Power consumption metrics.
///
/// ## Measurement Units
/// The units described here come from AMD SMI documentation for the `linux_bm` platform, which means "Linux bare-metal".
/// On other platforms, such as `host` (virtualization hypervisor), the units can change.
///
/// ## Unsupported Values
/// In the low-level AMD SMI, unsupported members are represented with special values.
/// On top of that, some fields did not exist in early versions of AMD SMI.
/// We represent both situations as `None`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AmdPowerInfo {
    /// Socket power in W.
    pub socket_power: Option<u64>,
    /// Current socket power in W, Mi 300+ Series cards.
    pub current_socket_power: Option<u32>,
    /// Average socket power in W, Navi + Mi 200 and earlier Series cards.
    pub average_socket_power: Option<u32>,

    /// GFX voltage measurement in mV.
    pub gfx_voltage: u64,
    /// SOC voltage measurement in mV.
    pub soc_voltage: u64,
    /// MEM voltage measurement in mV.
    pub mem_voltage: u64,

    /// The power limit in W.
    pub power_limit: u32,
}

impl AmdPowerInfo {
    pub fn get(handle: &AmdProcessorHandle) -> Result<Self, crate::error::AmdError> {
        let stable = &handle.amdsmi.shared.inner.lib_stable;
        let info = match &handle.amdsmi.shared.inner.lib_versioned {
            VersionedLib::V6_3_0(_) => {
                let mut info = MaybeUninit::<v6_3_0::amdsmi_power_info_t>::uninit();

                let res =
                    unsafe { stable.amdsmi_get_power_info(handle.inner, info.as_mut_ptr() as _) };
                handle.amdsmi.check_status(res)?;

                let info = unsafe { info.assume_init() };
                AmdPowerInfo::from(info)
            }
            VersionedLib::V6_4_0(_) | VersionedLib::V6_4_2(_) => {
                // amdsmi_power_info_t is the same in 6.4.0 and 6.4.2
                let mut info = MaybeUninit::<v6_4_0::amdsmi_power_info_t>::uninit();

                let res =
                    unsafe { stable.amdsmi_get_power_info(handle.inner, info.as_mut_ptr() as _) };
                handle.amdsmi.check_status(res)?;

                let info = unsafe { info.assume_init() };
                AmdPowerInfo::from(info)
            }
            VersionedLib::V7_0_0(_) | VersionedLib::V7_2_0(_) => {
                // amdsmi_power_info_t is the same in 7.0.0 and 7.2.0
                let mut info = MaybeUninit::<v7_0_0::amdsmi_power_info_t>::uninit();

                let res =
                    unsafe { stable.amdsmi_get_power_info(handle.inner, info.as_mut_ptr() as _) };
                handle.amdsmi.check_status(res)?;

                let info = unsafe { info.assume_init() };
                AmdPowerInfo::from(info)
            }
        };
        Ok(info)
    }
}

/// Filter out unsupported values.
///
/// # What value does AMD SMI use when it's unsupported?
/// The doc reads "Unsupported struct members are set to UINT32_MAX",
/// but the ROCm code uses 0xFFFF (u16::MAX), so...
fn maybe_unsupported(v: u32) -> Option<u32> {
    Some(v).filter(|v| *v != u32::MAX && *v != 0xFFFF)
}

impl From<v6_3_0::amdsmi_power_info_t> for AmdPowerInfo {
    fn from(info: v6_3_0::amdsmi_power_info_t) -> Self {
        Self {
            socket_power: None,
            current_socket_power: maybe_unsupported(info.current_socket_power),
            average_socket_power: maybe_unsupported(info.average_socket_power),
            gfx_voltage: info.gfx_voltage as u64,
            soc_voltage: info.soc_voltage as u64,
            mem_voltage: info.mem_voltage as u64,
            power_limit: info.power_limit,
        }
    }
}

impl From<v6_4_0::amdsmi_power_info_t> for AmdPowerInfo {
    fn from(info: v6_4_0::amdsmi_power_info_t) -> Self {
        Self {
            socket_power: None, // socket_power is host only in v6
            current_socket_power: maybe_unsupported(info.current_socket_power),
            average_socket_power: maybe_unsupported(info.average_socket_power),
            gfx_voltage: info.gfx_voltage as u64,
            soc_voltage: info.soc_voltage as u64,
            mem_voltage: info.mem_voltage as u64,
            power_limit: info.power_limit,
        }
    }
}

impl From<v7_0_0::amdsmi_power_info_t> for AmdPowerInfo {
    fn from(info: v7_0_0::amdsmi_power_info_t) -> Self {
        Self {
            socket_power: Some(info.socket_power),
            current_socket_power: maybe_unsupported(info.current_socket_power),
            average_socket_power: maybe_unsupported(info.average_socket_power),
            gfx_voltage: info.gfx_voltage,
            soc_voltage: info.soc_voltage,
            mem_voltage: info.mem_voltage,
            power_limit: info.power_limit,
        }
    }
}
