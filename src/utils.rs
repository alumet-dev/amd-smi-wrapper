use crate::bindings::*;
use bitflags::bitflags;

const CODE_SUCCESS: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_SUCCESS;
const CODE_INVAL: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_INVAL;
const CODE_NOT_SUPPORTED: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_NOT_SUPPORTED;
const CODE_OUT_OF_RESSOURCE: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_OUT_OF_RESOURCES;
const CODE_UNEXPECTED_DATA: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_UNEXPECTED_DATA;
const CODE_NO_PERM: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_NO_PERM;
const CODE_NOT_YET_IMPLEMENTED: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_NOT_YET_IMPLEMENTED;
const CODE_UNKNOWN_ERROR: amdsmi_status_t = amdsmi_status_t_AMDSMI_STATUS_UNKNOWN_ERROR;

bitflags! {
    /// List of all possible bitmask value to initialize AMD-SMI library for a given hardware type to analyze.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InitFlags: amdsmi_init_flags_t {
        const ALL_PROCESSORS = amdsmi_init_flags_t_AMDSMI_INIT_ALL_PROCESSORS;
        const AMD_CPUS = amdsmi_init_flags_t_AMDSMI_INIT_AMD_CPUS;
        const AMD_GPUS = amdsmi_init_flags_t_AMDSMI_INIT_AMD_GPUS;
        const AMD_APUS = amdsmi_init_flags_t_AMDSMI_INIT_AMD_APUS;
        const NON_AMD_CPUS = amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_CPUS;
        const NON_AMD_GPUS = amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_GPUS;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AmdEnergyConsumptionInfo {
    /// The energy consumption value of an AMD GPU device since the last boot in micro Joules.
    pub energy: u64,
    /// Precision factor of the energy counter in micro Joules.
    pub resolution: f32,
    /// The time during which the energy value is recovered in ns.
    pub timestamp: u64,
}

/// List of all possible status and return code for AMD-SMI library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AmdStatus {
    Success = CODE_SUCCESS,
    Inval = CODE_INVAL,
    NotSupported = CODE_NOT_SUPPORTED,
    NotYetImplemented = amdsmi_status_t_AMDSMI_STATUS_NOT_YET_IMPLEMENTED,
    FailLoadModule = amdsmi_status_t_AMDSMI_STATUS_FAIL_LOAD_MODULE,
    FailLoadSymbol = amdsmi_status_t_AMDSMI_STATUS_FAIL_LOAD_SYMBOL,
    DrmError = amdsmi_status_t_AMDSMI_STATUS_DRM_ERROR,
    ApiFailed = amdsmi_status_t_AMDSMI_STATUS_API_FAILED,
    Timeout = amdsmi_status_t_AMDSMI_STATUS_TIMEOUT,
    Retry = amdsmi_status_t_AMDSMI_STATUS_RETRY,
    NoPerm = amdsmi_status_t_AMDSMI_STATUS_NO_PERM,
    Interrupt = amdsmi_status_t_AMDSMI_STATUS_INTERRUPT,
    Io = amdsmi_status_t_AMDSMI_STATUS_IO,
    AddressFault = amdsmi_status_t_AMDSMI_STATUS_ADDRESS_FAULT,
    FileError = amdsmi_status_t_AMDSMI_STATUS_FILE_ERROR,
    OutOfResources = CODE_OUT_OF_RESSOURCE,
    InternalException = amdsmi_status_t_AMDSMI_STATUS_INTERNAL_EXCEPTION,
    InputOutOfBounds = amdsmi_status_t_AMDSMI_STATUS_INPUT_OUT_OF_BOUNDS,
    InitError = amdsmi_status_t_AMDSMI_STATUS_INIT_ERROR,
    RefcountOverflow = amdsmi_status_t_AMDSMI_STATUS_REFCOUNT_OVERFLOW,
    DirectoryNotFound = amdsmi_status_t_AMDSMI_STATUS_DIRECTORY_NOT_FOUND,
    Busy = amdsmi_status_t_AMDSMI_STATUS_BUSY,
    NotFound = amdsmi_status_t_AMDSMI_STATUS_NOT_FOUND,
    NotInit = amdsmi_status_t_AMDSMI_STATUS_NOT_INIT,
    NoSlot = amdsmi_status_t_AMDSMI_STATUS_NO_SLOT,
    DriverNotLoaded = amdsmi_status_t_AMDSMI_STATUS_DRIVER_NOT_LOADED,
    MoreData = amdsmi_status_t_AMDSMI_STATUS_MORE_DATA,
    NoData = amdsmi_status_t_AMDSMI_STATUS_NO_DATA,
    InsufficientSize = amdsmi_status_t_AMDSMI_STATUS_INSUFFICIENT_SIZE,
    UnexpectedSize = amdsmi_status_t_AMDSMI_STATUS_UNEXPECTED_SIZE,
    UnexpectedData = amdsmi_status_t_AMDSMI_STATUS_UNEXPECTED_DATA,
    NonAmdCpu = amdsmi_status_t_AMDSMI_STATUS_NON_AMD_CPU,
    NoEnergyDrv = amdsmi_status_t_AMDSMI_STATUS_NO_ENERGY_DRV,
    NoMsrDrv = amdsmi_status_t_AMDSMI_STATUS_NO_MSR_DRV,
    NoHsmpDrv = amdsmi_status_t_AMDSMI_STATUS_NO_HSMP_DRV,
    NoHsmpSup = amdsmi_status_t_AMDSMI_STATUS_NO_HSMP_SUP,
    NoHsmpMsgSup = amdsmi_status_t_AMDSMI_STATUS_NO_HSMP_MSG_SUP,
    HsmpTimeout = amdsmi_status_t_AMDSMI_STATUS_HSMP_TIMEOUT,
    NoDrv = amdsmi_status_t_AMDSMI_STATUS_NO_DRV,
    FileNotFound = amdsmi_status_t_AMDSMI_STATUS_FILE_NOT_FOUND,
    ArgPtrNull = amdsmi_status_t_AMDSMI_STATUS_ARG_PTR_NULL,
    AmdgpuRestartErr = amdsmi_status_t_AMDSMI_STATUS_AMDGPU_RESTART_ERR,
    SettingUnavailable = amdsmi_status_t_AMDSMI_STATUS_SETTING_UNAVAILABLE,
    CorruptedEeprom = amdsmi_status_t_AMDSMI_STATUS_CORRUPTED_EEPROM,
    MapError = amdsmi_status_t_AMDSMI_STATUS_MAP_ERROR,
    UnknownError = CODE_UNKNOWN_ERROR,
}

impl From<amdsmi_status_t> for AmdStatus {
    fn from(status: amdsmi_status_t) -> Self {
        match status {
            CODE_SUCCESS => AmdStatus::Success,
            CODE_INVAL => AmdStatus::Inval,
            CODE_NOT_SUPPORTED => AmdStatus::NotSupported,
            CODE_OUT_OF_RESSOURCE => AmdStatus::OutOfResources,
            CODE_NO_PERM => AmdStatus::NoPerm,
            CODE_NOT_YET_IMPLEMENTED => AmdStatus::NotYetImplemented,
            CODE_UNEXPECTED_DATA => AmdStatus::UnexpectedData,
            _ => AmdStatus::UnknownError,
        }
    }
}
