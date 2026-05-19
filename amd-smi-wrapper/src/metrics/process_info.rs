use std::mem::MaybeUninit;

use amd_smi_wrapper_sys::{
    load::VersionedLib,
    versions::{
        stable::{self, AMDSMI_STATUS_OUT_OF_RESOURCES, AMDSMI_STATUS_SUCCESS},
        v6_3_0, v6_4_0, v6_4_2, v7_2_0,
    },
};

use crate::{handles::AmdProcessorHandle, utils::c_buffer_to_string};

/// Statistics about a running process, see `amdsmi_proc_info_t`.
#[derive(Debug, Default, Clone)]
pub struct AmdProcessInfo {
    /// ASCII path name of the process.
    pub name: String,
    /// process ID.
    pub pid: u32,
    /// Process memory usage in Bytes.
    pub mem: u64,
    pub engine_usage: AmdProcessEngineUsage,
    pub memory_usage: AmdProcessMemoryUsage,
    /// ASCII name of the process or container.
    pub container_name: String,
    /// Number of compute units utilized.
    pub cu_occupancy: Option<u32>,
    /// Time that queues are evicted on a GPU in milliseconds.
    pub evicted_time: Option<u32>,
}

/// Parameters about engine activity usage by process, see `amdsmi_proc_info_t_memory_usage_`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AmdProcessEngineUsage {
    /// Process graphic core unit usage in nanoseconds.
    pub gfx: u64,
    /// Encoding units usage in nanoseconds.
    pub enc: u64,
}

/// Parameters about consumed memory by process, see `amdsmi_proc_info_t_memory_usage_`.
#[derive(Debug, Default, Clone)]
pub struct AmdProcessMemoryUsage {
    /// Process GTT memory usage in Bytes.
    pub gtt_mem: u64,
    /// Process CPU memory usage in Bytes.
    pub cpu_mem: u64,
    /// Process VRAM memory usage in Bytes.
    pub vram_mem: u64,
}

impl AmdProcessInfo {
    pub fn list(handle: &AmdProcessorHandle) -> Result<Vec<Self>, crate::error::AmdError> {
        enum ProcList {
            V6_3_0(Vec<MaybeUninit<v6_3_0::amdsmi_proc_info_t>>),
            V6_4_0(Vec<MaybeUninit<v6_4_0::amdsmi_proc_info_t>>),
            V6_4_2(Vec<MaybeUninit<v6_4_2::amdsmi_proc_info_t>>),
            V7_2_0(Vec<MaybeUninit<v7_2_0::amdsmi_proc_info_t>>),
        }

        impl ProcList {
            fn with_capacity(lib: &VersionedLib, capacity: usize) -> Self {
                match lib {
                    VersionedLib::V6_3_0(_) => ProcList::V6_3_0(Vec::with_capacity(capacity)),
                    VersionedLib::V6_4_0(_) => ProcList::V6_4_0(Vec::with_capacity(capacity)),
                    VersionedLib::V6_4_2(_) => ProcList::V6_4_2(Vec::with_capacity(capacity)),
                    VersionedLib::V7_0_0(_) => ProcList::V6_4_2(Vec::with_capacity(capacity)),
                    VersionedLib::V7_2_0(_) => ProcList::V7_2_0(Vec::with_capacity(capacity)),
                }
            }

            unsafe fn set_len(&mut self, len: usize) {
                match self {
                    ProcList::V6_3_0(v) => unsafe { v.set_len(len) },
                    ProcList::V6_4_0(v) => unsafe { v.set_len(len) },
                    ProcList::V6_4_2(v) => unsafe { v.set_len(len) },
                    ProcList::V7_2_0(v) => unsafe { v.set_len(len) },
                }
            }

            fn reserve_exact(&mut self, cap: usize) {
                match self {
                    ProcList::V6_3_0(v) => v.reserve_exact(cap),
                    ProcList::V6_4_0(v) => v.reserve_exact(cap),
                    ProcList::V6_4_2(v) => v.reserve_exact(cap),
                    ProcList::V7_2_0(v) => v.reserve_exact(cap),
                }
            }

            fn as_mut_ptr(&mut self) -> *mut stable::amdsmi_proc_info_t {
                match self {
                    ProcList::V6_3_0(v) => v.as_mut_ptr() as _,
                    ProcList::V6_4_0(v) => v.as_mut_ptr() as _,
                    ProcList::V6_4_2(v) => v.as_mut_ptr() as _,
                    ProcList::V7_2_0(v) => v.as_mut_ptr() as _,
                }
            }

            fn into_high_level(self) -> Vec<AmdProcessInfo> {
                match self {
                    ProcList::V6_3_0(v) => v
                        .into_iter()
                        .map(|p| AmdProcessInfo::from(unsafe { p.assume_init() }))
                        .collect(),
                    ProcList::V6_4_0(v) => v
                        .into_iter()
                        .map(|p| AmdProcessInfo::from(unsafe { p.assume_init() }))
                        .collect(),
                    ProcList::V6_4_2(v) => v
                        .into_iter()
                        .map(|p| AmdProcessInfo::from(unsafe { p.assume_init() }))
                        .collect(),
                    ProcList::V7_2_0(v) => v
                        .into_iter()
                        .map(|p| AmdProcessInfo::from(unsafe { p.assume_init() }))
                        .collect(),
                }
            }
        }

        const MAX_TRIES: usize = 5;
        const INITIAL_CAPACITY: usize = 32;

        let mut list =
            ProcList::with_capacity(&handle.amdsmi.shared.inner.lib_versioned, INITIAL_CAPACITY);
        let mut count = INITIAL_CAPACITY as u32;
        let mut tries = 0;
        loop {
            let res = unsafe {
                handle
                    .amdsmi
                    .shared
                    .inner
                    .lib_stable
                    .amdsmi_get_gpu_process_list(handle.inner, &mut count, list.as_mut_ptr())
            };
            match res {
                AMDSMI_STATUS_SUCCESS => {
                    unsafe { list.set_len(count as usize) };
                    break;
                }
                AMDSMI_STATUS_OUT_OF_RESOURCES => {
                    list.reserve_exact(count as usize);
                }
                err => return Err(handle.amdsmi.build_error(err)),
            }
            tries += 1;
            if tries > MAX_TRIES {
                log::warn!(
                    "Tried {MAX_TRIES} times but the number of process always changed. Stopping here."
                );
                break;
            }
        }
        Ok(list.into_high_level())
    }
}

impl From<v6_3_0::amdsmi_proc_info_t> for AmdProcessInfo {
    fn from(info: v6_3_0::amdsmi_proc_info_t) -> Self {
        Self {
            name: c_buffer_to_string(&info.name),
            pid: info.pid,
            mem: info.mem,
            engine_usage: AmdProcessEngineUsage {
                gfx: info.engine_usage.gfx,
                enc: info.engine_usage.enc,
            },
            memory_usage: AmdProcessMemoryUsage {
                gtt_mem: info.memory_usage.gtt_mem,
                cpu_mem: info.memory_usage.cpu_mem,
                vram_mem: info.memory_usage.vram_mem,
            },
            container_name: c_buffer_to_string(&info.container_name),
            cu_occupancy: None,
            evicted_time: None,
        }
    }
}

impl From<v6_4_0::amdsmi_proc_info_t> for AmdProcessInfo {
    fn from(info: v6_4_0::amdsmi_proc_info_t) -> Self {
        Self {
            name: c_buffer_to_string(&info.name),
            pid: info.pid,
            mem: info.mem,
            engine_usage: AmdProcessEngineUsage {
                gfx: info.engine_usage.gfx,
                enc: info.engine_usage.enc,
            },
            memory_usage: AmdProcessMemoryUsage {
                gtt_mem: info.memory_usage.gtt_mem,
                cpu_mem: info.memory_usage.cpu_mem,
                vram_mem: info.memory_usage.vram_mem,
            },
            container_name: c_buffer_to_string(&info.container_name),
            cu_occupancy: None,
            evicted_time: None,
        }
    }
}
impl From<v6_4_2::amdsmi_proc_info_t> for AmdProcessInfo {
    fn from(info: v6_4_2::amdsmi_proc_info_t) -> Self {
        Self {
            name: c_buffer_to_string(&info.name),
            pid: info.pid,
            mem: info.mem,
            engine_usage: AmdProcessEngineUsage {
                gfx: info.engine_usage.gfx,
                enc: info.engine_usage.enc,
            },
            memory_usage: AmdProcessMemoryUsage {
                gtt_mem: info.memory_usage.gtt_mem,
                cpu_mem: info.memory_usage.cpu_mem,
                vram_mem: info.memory_usage.vram_mem,
            },
            container_name: c_buffer_to_string(&info.container_name),
            cu_occupancy: Some(info.cu_occupancy),
            evicted_time: None,
        }
    }
}

impl From<v7_2_0::amdsmi_proc_info_t> for AmdProcessInfo {
    fn from(info: v7_2_0::amdsmi_proc_info_t) -> Self {
        Self {
            name: c_buffer_to_string(&info.name),
            pid: info.pid,
            mem: info.mem,
            engine_usage: AmdProcessEngineUsage {
                gfx: info.engine_usage.gfx,
                enc: info.engine_usage.enc,
            },
            memory_usage: AmdProcessMemoryUsage {
                gtt_mem: info.memory_usage.gtt_mem,
                cpu_mem: info.memory_usage.cpu_mem,
                vram_mem: info.memory_usage.vram_mem,
            },
            container_name: c_buffer_to_string(&info.container_name),
            cu_occupancy: Some(info.cu_occupancy),
            evicted_time: Some(info.evicted_time),
        }
    }
}
