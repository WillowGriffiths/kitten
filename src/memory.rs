#[derive(Clone, Copy, Debug)]
pub struct MemoryRange {
    pub start: u64,
    pub len: u64,
}

impl MemoryRange {
    pub fn new(start: u64, len: u64) -> Self {
        Self { start, len }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryMapping {
    pub phys: u64,
    pub virt: u64,
    pub len: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryInfo {
    pub memory: MemoryMapping,
    pub kernel: MemoryMapping,
}

static mut MEMORY_INFO: Option<MemoryInfo> = None;

pub fn set_memory_info(value: MemoryInfo) {
    unsafe {
        MEMORY_INFO = Some(value);
    }
}

pub fn to_phys(value: u64) -> u64 {
    let memory_info = unsafe { MEMORY_INFO.expect("uninitialised memory info") };

    let memory_end = memory_info.memory.virt + memory_info.memory.len;
    let kernel_end = memory_info.kernel.virt + memory_info.kernel.len;

    if value >= memory_info.memory.virt && value < memory_end {
        value - memory_info.memory.virt + memory_info.memory.phys
    } else if value >= memory_info.kernel.virt && value < kernel_end {
        value - memory_info.kernel.virt + memory_info.kernel.phys
    } else {
        panic!("Invalid memory address");
    }
}
