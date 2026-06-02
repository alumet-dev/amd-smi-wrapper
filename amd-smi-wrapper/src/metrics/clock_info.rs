use std::mem::MaybeUninit;

use amd_smi_wrapper_sys::versions::stable;

use crate::handles::AmdProcessorHandle;

/// GPU clock metrics.
#[derive(Debug, Default, Clone)]
pub struct AmdClockInfo {
    /// Clock frequency in MHz.
    pub clk: u32,
    /// Minimal clock frequency in MHz.
    pub min_clk: u32,
    /// Maximal clock frequency in MHz.
    pub max_clk: u32,
    /// Clock locked status boolean status
    pub clk_locked: u8,
    /// Clock deep sleep status boolean status
    pub clk_deep_sleep: u8,
}

impl AmdClockInfo {
    pub fn get(
        handle: &AmdProcessorHandle,
        clk_type: stable::amdsmi_clk_type_t,
    ) -> Result<Self, crate::error::AmdError> {
        let mut info = MaybeUninit::<stable::amdsmi_clk_info_t>::uninit();

        // SAFETY: Pass a pointer to uninitialized memory to the FFI function.
        // According to AMD-SMI documentation, the function fully initializes the `amdsmi_clk_info_t` on success.
        // The `SUCCESS` return code `amdsmi_status_t` is checked before using the data.
        let result = unsafe {
            handle.amdsmi.shared.inner.lib_stable.amdsmi_get_clock_info(
                handle.inner,
                clk_type,
                info.as_mut_ptr(),
            )
        };

        handle.amdsmi.check_status(result)?;

        // SAFETY: `assume_init()` is safe because the FFI call succeeded and the structure was fully initialized by the library.
        let info = unsafe { info.assume_init() };
        Ok(info.into())
    }
}

impl From<stable::amdsmi_clk_info_t> for AmdClockInfo {
    fn from(value: stable::amdsmi_clk_info_t) -> Self {
        Self {
            clk: value.clk,
            min_clk: value.min_clk,
            max_clk: value.max_clk,
            clk_locked: value.clk_locked,
            clk_deep_sleep: value.clk_deep_sleep,
        }
    }
}
