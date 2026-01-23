use crate::arch::println;
use crate::arch::riscv::pagetable;
use crate::device_tree::{FdtInfo, FdtNode, FdtNodeChild};
use crate::memory::{self, MemoryInfo, MemoryMapping, MemoryRange};
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(hart_id: u64, fdt: *const u8, kernel_start: u64) -> ! {
    println!("We are running on hart {hart_id}");
    let fdt = unsafe { fdt.sub(0x80000000).add(0xffffffff80000000) };

    let fdt_info = FdtInfo::new(fdt);
    let boot_info = boot_info(&fdt_info, kernel_start);

    memory::set_memory_info(boot_info.memory_info);

    pagetable::setup(&boot_info.memory_info);

    crate::main(boot_info);
}

#[derive(Debug)]
pub struct BootInfo {
    pub memory_info: MemoryInfo,
    pub resv: Option<(usize, [MemoryRange; 16])>,
    pub cpus: usize,
}

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

unsafe extern "C" {
    #[link_name = "__virtual_kernel_start"]
    static KERNEL_START: u8;
    #[link_name = "__virtual_end"]
    static KERNEL_END: u8;
}

fn boot_info(fdt_info: &FdtInfo, kernel_start: u64) -> BootInfo {
    let mut memory: Option<MemoryRange> = None;
    let mut resv = None;
    let mut cpus = 0;

    for child in fdt_info.root_node() {
        if let FdtNodeChild::Node(mut node) = child {
            if node.name.starts_with("memory@") {
                if memory.is_some() {
                    panic!("only one memory range is supported");
                }

                memory = Some(parse_memory(&mut node));
            } else if node.name == "reserved-memory" {
                if memory.is_some() {
                    panic!("multiple reserved-memory nodes");
                }

                resv = Some(parse_reserved_memory(&mut node));
            } else if node.name == "cpus" {
                cpus = parse_cpus(&mut node);
            }
        }
    }

    let kernel_mapping = unsafe {
        let kernel_start_addr = (&KERNEL_START as *const u8) as u64;
        let kernel_end_addr = (&KERNEL_END as *const u8) as u64;
        let kernel_size = kernel_end_addr - kernel_start_addr;

        MemoryMapping {
            phys: kernel_start,
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

    BootInfo {
        memory_info: MemoryInfo {
            memory: memory_mapping,
            kernel: kernel_mapping,
        },
        resv,
        cpus,
    }
}
