use alloc::boxed::Box;

use crate::arch::riscv::pagetable;
use crate::device_tree::{FdtInfo, FdtNode, FdtNodeChild};
use crate::memory::{self, MemoryInfo, MemoryMapping, MemoryRange};
use crate::multicore::BootRes;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

unsafe extern "C" {
    static _kernel_start_addr: usize;
}

#[unsafe(no_mangle)]
extern "C" fn rust_entry(hart_id: u64, fdt_addr: usize) -> ! {
    let boot_info = BootInfo::new(fdt_addr, hart_id);

    memory::set_memory_info(boot_info.memory_info);

    pagetable::setup(&boot_info.memory_info);

    crate::main(crate::BootData::Primary(boot_info));
}

#[unsafe(no_mangle)]
extern "C" fn secondary_rust_entry(boot_res_phys: usize) {
    let boot_res_virt = memory::to_virt(boot_res_phys as u64) as *mut BootRes;
    let boot_res = unsafe { Box::from_raw(boot_res_virt) };

    crate::main(crate::BootData::Secondary(*boot_res));
}

#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub memory_info: MemoryInfo,
    pub resv_count: usize,
    pub resv: [MemoryRange; 16],
    pub cpus: usize,

    pub fdt_addr: usize,

    pub boot_cpu: u64,
}

unsafe extern "C" {
    #[link_name = "__virtual_kernel_start"]
    static KERNEL_START: u8;
    #[link_name = "__virtual_end"]
    static KERNEL_END: u8;
}

impl BootInfo {
    fn parse_memory(node: &mut FdtNode) -> MemoryRange {
        for child in node {
            if let FdtNodeChild::Prop(name, data) = child
                && name == "reg"
            {
                let ranges = data.len() / 16;
                if ranges != 1 {
                    panic!("only one memory range is supported");
                }
                let start = u64::from_be_bytes(data[0..8].try_into().unwrap());
                let len = u64::from_be_bytes(data[8..16].try_into().unwrap());

                return MemoryRange::new(start, len);
            }
        }

        panic!("No range found");
    }

    fn parse_cpus(node: &mut FdtNode) -> usize {
        let mut cpus = 0;
        for child in node {
            if let FdtNodeChild::Node(node) = child
                && node.name.starts_with("cpu@")
            {
                cpus += 1;
            }
        }

        cpus
    }

    fn parse_reserved_memory(node: &mut FdtNode) -> (usize, [MemoryRange; 16]) {
        let mut resv = [MemoryRange::new(0, 0); 16];
        let mut resv_count = 0;

        for child in node {
            if let FdtNodeChild::Node(node) = child {
                for child in node {
                    if let FdtNodeChild::Prop(name, data) = child
                        && name == "reg"
                    {
                        let ranges = data.len() / 16;
                        for i in 0..ranges {
                            let start_index = 16 * i;
                            let start = u64::from_be_bytes(
                                data[start_index..start_index + 8].try_into().unwrap(),
                            );
                            let len = u64::from_be_bytes(
                                data[start_index + 8..start_index + 16].try_into().unwrap(),
                            );

                            resv[resv_count] = MemoryRange::new(start, len);
                            resv_count += 1;
                        }
                    }
                }
            }
        }

        (resv_count, resv)
    }

    fn new(fdt_addr: usize, hart_id: u64) -> BootInfo {
        let fdt = (fdt_addr - 0x80000000 + 0xffffffff80000000) as *const u8;
        let fdt_info = FdtInfo::new(fdt);

        let mut memory: Option<MemoryRange> = None;
        let mut resv = None;
        let mut cpus = 0;

        for child in fdt_info.root_node() {
            if let FdtNodeChild::Node(mut node) = child {
                if node.name.starts_with("memory@") {
                    if memory.is_some() {
                        panic!("only one memory range is supported");
                    }

                    memory = Some(Self::parse_memory(&mut node));
                } else if node.name == "reserved-memory" {
                    if resv.is_some() {
                        panic!("multiple reserved-memory nodes");
                    }

                    resv = Some(Self::parse_reserved_memory(&mut node));
                } else if node.name == "cpus" {
                    cpus = Self::parse_cpus(&mut node);
                }
            }
        }

        let kernel_mapping = unsafe {
            let kernel_start_addr = (&KERNEL_START as *const u8) as u64;
            let kernel_end_addr = (&KERNEL_END as *const u8) as u64;
            let kernel_size = kernel_end_addr - kernel_start_addr;

            MemoryMapping {
                phys: _kernel_start_addr as u64,
                virt: kernel_start_addr,
                len: kernel_size,
            }
        };

        let memory = memory.expect("Found no memory");

        let memory_mapping = MemoryMapping {
            phys: memory.start,
            virt: 0xffffffde80000000,
            len: memory.len,
        };

        let resv = resv.unwrap_or((0, [MemoryRange { start: 0, len: 0 }; 16]));

        let fdt_memory = fdt_info.memory_range();
        let fdt_memory_physical = MemoryRange {
            start: fdt_memory.start - kernel_mapping.virt + kernel_mapping.phys,
            len: fdt_memory.len,
        };

        let resv_count = resv.0 + 2;
        let mut resv = resv.1;
        if resv_count > resv.len() {
            panic!("Too many reserved sections");
        }

        resv[resv_count - 2] = fdt_memory_physical;
        resv[resv_count - 1] = MemoryRange {
            start: kernel_mapping.phys,
            len: kernel_mapping.len,
        };

        BootInfo {
            memory_info: MemoryInfo {
                memory: memory_mapping,
                kernel: kernel_mapping,
            },
            resv_count,
            resv,
            cpus,

            fdt_addr,

            boot_cpu: hart_id,
        }
    }
}
