use core::{ffi::CStr, slice};

use crate::arch::println;

#[derive(Debug)]
enum FdtToken {
    NodeBegin(FdtNode),
    NodeEnd,
    Prop(&'static str, &'static [u8]),
}

pub struct FdtInfo {
    dt_struct: *const u8,
    dt_strings: *const u8,
}

#[derive(Debug)]
pub struct FdtNode {
    pub name: &'static str,
    dt_strings: *const u8,
    dt_struct: *const u8,
}

impl FdtInfo {
    pub fn new(fdt: *const u8) -> FdtInfo {
        unsafe {
            let header = fdt as *const u32;
            let magic = u32::from_be(*header);
            if magic != 0xd00dfeed {
                panic!("Bad magic");
            }

            let compatible_version = u32::from_be(*header.add(6));
            if compatible_version > 17 {
                panic!("Bad version");
            }

            let total_size = u32::from_be(*header.add(1));
            println!("found compatible device tree, total size: {total_size}");

            let dt_struct_offset = u32::from_be(*header.add(2));
            let dt_strings_offset = u32::from_be(*header.add(3));

            let dt_struct = fdt.add(dt_struct_offset as usize);
            let dt_strings = fdt.add(dt_strings_offset as usize);

            FdtInfo {
                dt_struct,
                dt_strings,
            }
        }
    }

    pub fn root_node(&self) -> FdtNode {
        unsafe {
            let mut ptr = self.dt_struct;
            loop {
                let token = u32::from_be(*(ptr as *const u32));
                match token {
                    FDT_NOP => {}
                    FDT_BEGIN_NODE => {
                        let name_c: &'static CStr = CStr::from_ptr(ptr.add(4));
                        let name = name_c.to_str().expect("invalid node name");
                        let bytes = name.len();

                        ptr = ptr.add(4 + bytes + 1);
                        ptr = ptr.add(ptr.align_offset(4));

                        let node = FdtNode {
                            name,
                            dt_struct: ptr,
                            dt_strings: self.dt_strings,
                        };

                        return node;
                    }
                    _ => panic!("unexpected token"),
                };

                ptr = ptr.add(4);
            }
        }
    }
}

const FDT_BEGIN_NODE: u32 = 0x01;
const FDT_END_NODE: u32 = 0x02;
const FDT_PROP: u32 = 0x03;
const FDT_NOP: u32 = 0x04;
const FDT_END: u32 = 0x09;

impl FdtNode {
    fn consume_token(&mut self) -> Option<FdtToken> {
        unsafe {
            let mut token = u32::from_be(*(self.dt_struct as *const u32));

            while token == FDT_NOP {
                self.dt_struct = self.dt_struct.add(4);
                token = u32::from_be(*(self.dt_struct as *const u32));
            }

            match token {
                FDT_BEGIN_NODE => {
                    let name_c: &'static CStr = CStr::from_ptr(self.dt_struct.add(4));
                    let name = name_c.to_str().expect("invalid node name");
                    let bytes = name.len();
                    self.dt_struct = self.dt_struct.add(4 + bytes + 1);
                    self.dt_struct = self.dt_struct.add(self.dt_struct.align_offset(4));

                    let node = FdtNode {
                        name,
                        dt_struct: self.dt_struct,
                        dt_strings: self.dt_strings,
                    };

                    Some(FdtToken::NodeBegin(node))
                }
                FDT_END_NODE => {
                    self.dt_struct = self.dt_struct.add(4);

                    Some(FdtToken::NodeEnd)
                }
                FDT_PROP => {
                    let len = u32::from_be(*(self.dt_struct.add(4) as *const u32));
                    let name_off = u32::from_be(*(self.dt_struct.add(8) as *const u32));
                    let name_c: &'static CStr =
                        CStr::from_ptr(self.dt_strings.add(name_off as usize));
                    let name = name_c.to_str().expect("Invalid property name");
                    let data: &'static [u8] =
                        slice::from_raw_parts(self.dt_struct.add(12), len as usize);

                    self.dt_struct = self.dt_struct.add(12 + len as usize);
                    self.dt_struct = self.dt_struct.add(self.dt_struct.align_offset(4));

                    Some(FdtToken::Prop(name, data))
                }
                FDT_END => None,
                _ => {
                    panic!("unknown token!");
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum FdtNodeChild {
    Node(FdtNode),
    Prop(&'static str, &'static [u8]),
}

impl Iterator for FdtNode {
    type Item = FdtNodeChild;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.consume_token() {
            return match token {
                FdtToken::NodeBegin(node) => {
                    let mut depth = 1;
                    while depth > 0 {
                        let token = self.consume_token().unwrap();

                        match token {
                            FdtToken::NodeBegin(_) => depth += 1,
                            FdtToken::NodeEnd => depth -= 1,
                            _ => {}
                        }
                    }

                    Some(FdtNodeChild::Node(node))
                }
                FdtToken::NodeEnd => unsafe {
                    self.dt_struct = self.dt_struct.sub(4);
                    None
                },
                FdtToken::Prop(name, data) => Some(FdtNodeChild::Prop(name, data)),
            };
        }

        panic!("reached end of FDT before node ended");
    }
}
