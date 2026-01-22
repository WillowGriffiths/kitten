use core::ffi::CStr;

use crate::arch::{print, println};

pub fn parse(fdt: *const u32) {
    unsafe {
        let magic = u32::from_be(*fdt);
        if magic != 0xd00dfeed {
            panic!("Bad magic");
        }

        let compatible_version = u32::from_be(*fdt.add(6));
        if compatible_version > 17 {
            panic!("Bad version");
        }

        let total_size = u32::from_be(*fdt.add(1));
        println!("found compatible device tree, total size: {total_size}");

        let addr = fdt as *const u8;
        let dt_struct_offset = u32::from_be(*fdt.add(2));
        let dt_strings_offset = u32::from_be(*fdt.add(3));
        let mem_rsvmap_offset = u32::from_be(*fdt.add(4));

        let mem_rsvmap = addr.add(mem_rsvmap_offset as usize) as *const u64;
        let mut i = 0;
        loop {
            let address = u64::from_be(*mem_rsvmap.add(2 * i));
            let size = u64::from_be(*mem_rsvmap.add(2 * i + 1));

            if (address, size) == (0, 0) {
                break;
            }

            println!("Reserved memory: {address}, {size}");

            i += 1;
        }

        println!("found {i} reserved memory sections");

        let mut dt_struct = addr.add(dt_struct_offset as usize);
        let dt_strings = addr.add(dt_strings_offset as usize);

        let mut depth = 0;
        loop {
            let token = u32::from_be(*(dt_struct as *const u32));
            dt_struct = dt_struct.add(4);

            if token == 0x02 {
                depth -= 1;
            }
            if token != 0x04 {
                for _ in 0..depth * 2 {
                    print!(" ");
                }
            }
            if token == 0x01 {
                depth += 1;
            }

            match token {
                0x01 => {
                    let str = CStr::from_ptr(dt_struct);
                    let bytes = str.count_bytes();
                    println!("node {str:?} {{");
                    dt_struct = dt_struct.add(bytes + 1);
                }
                0x02 => {
                    println!("}}");
                }
                0x03 => {
                    let len = u32::from_be(*(dt_struct as *const u32));
                    let name_off = u32::from_be(*(dt_struct.add(4) as *const u32));
                    let name = CStr::from_ptr(dt_strings.add(name_off as usize));

                    println!("prop {name:?}, {len}B");

                    dt_struct = dt_struct.add(8 + len as usize);
                }
                0x04 => {}
                0x09 => {
                    println!("end of dt!");
                    break;
                }
                _ => {
                    println!("unknown token!");
                }
            }

            dt_struct = dt_struct.add(dt_struct.align_offset(4));
        }
    }
}
