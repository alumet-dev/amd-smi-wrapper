//! Metrics data structure and multi-version helpers.
use amd_smi_wrapper_sys::versions::stable;

pub mod asic_info;
pub mod clock_info;
pub mod power_info;
pub mod process_info;

pub type AmdClkType = stable::amdsmi_clk_type_t;
pub type AmdMemoryType = stable::amdsmi_memory_type_t;
pub type AmdTemperatureMetric = stable::amdsmi_temperature_metric_t;
pub type AmdTemperatureType = stable::amdsmi_temperature_type_t;
pub type AmdVoltageMetric = stable::amdsmi_voltage_metric_t;
pub type AmdVoltageType = stable::amdsmi_voltage_type_t;

/// Parameters about energy consumption of a GPU.
#[derive(Debug, Default, Clone, Copy)]
pub struct AmdEnergyConsumption {
    /// The energy consumption value of an AMD GPU device since the last boot in micro Joules.
    pub energy: u64,
    /// Precision factor of the energy counter in micro Joules.
    pub resolution: f32,
    /// The time during which the energy value is recovered in ns.
    pub timestamp: u64,
}

/// Parameters about the engine activity usage.
#[derive(Debug, Default, Clone, Copy)]
pub struct AmdEngineUsage {
    /// Main graphic core of AMD GPU, in percentage.
    pub gfx_activity: u32,
    /// Manage memory access and addresses translation, in percentage.
    pub mm_activity: u32,
    /// Memory controller managing access to VRAM in organizing writing/reading operations, in percentage.
    pub umc_activity: u32,
}

impl From<stable::amdsmi_engine_usage_t> for AmdEngineUsage {
    fn from(info: stable::amdsmi_engine_usage_t) -> Self {
        Self {
            gfx_activity: info.gfx_activity,
            mm_activity: info.mm_activity,
            umc_activity: info.umc_activity,
        }
    }
}

/// Parameters about PCI bus traffic by a GPU.
#[derive(Debug, Default, Clone, Copy)]
pub struct AmdPciTraffic {
    /// Number of bytes sent.
    pub sent: u64,
    /// Number of bytes received.
    pub received: u64,
    /// Maximum packet size.
    pub max_pkt_sz: u64,
}
