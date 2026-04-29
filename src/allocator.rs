use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    cell::UnsafeCell,
    mem,
    ptr::{self, NonNull},
};

use alloc::alloc::AllocError;

use crate::{
    arch::boot::BootInfo,
    memory::{self, Page, ZEROED_PAGE},
};

struct BuddyAllocator {
    data: UnsafeCell<Option<BuddyAllocatorData>>,
}

struct BuddyAllocatorData {
    slab: SlabAllocator,
    root_node: *mut BuddyNode,
    max_order: usize,
    can_alloc: bool,
}

unsafe impl Sync for BuddyAllocator {}

#[derive(Default)]
enum BuddyNode {
    #[default]
    Unallocated,
    Allocated,
    Branch(*mut BuddyNode, *mut BuddyNode),
}

static mut BUDDY_SLAB_PAGE: Page = ZEROED_PAGE;

const BUDDY_ORDER0: usize = memory::PAGE_SIZE;

impl BuddyAllocator {
    fn init(&self, boot_info: &BootInfo) {
        if unsafe { (*self.data.get()).is_some() } {
            panic!("Reinitialising buddy allocator");
        }

        let slab =
            unsafe { SlabAllocator::new_no_alloc::<BuddyNode>(&raw mut BUDDY_SLAB_PAGE, false) };

        let mut max_order = 0;

        while BUDDY_ORDER0 * 2_usize.pow(max_order) < boot_info.memory_info.memory.len as usize {
            max_order += 1;
        }

        let root_node = slab
            .allocate_should_recurse(Layout::new::<BuddyNode>(), false)
            .unwrap()
            .as_ptr() as *mut BuddyNode;

        unsafe { *root_node = BuddyNode::Unallocated };

        let data = BuddyAllocatorData {
            slab,
            root_node,
            max_order: max_order as usize,
            can_alloc: false,
        };

        unsafe {
            *self.data.get() = Some(data);
        }

        for memory::MemoryRange { start, len } in &boot_info.resv[0..boot_info.resv_count] {
            self.reserve_range(*start as usize, *len as usize);
        }

        unsafe {
            self.data
                .get()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap()
                .can_alloc = true
        }
    }

    fn reserve_range(&self, start: usize, len: usize) {
        let data = unsafe { self.data.get().as_mut().unwrap().as_mut().unwrap() };

        let start_aligned = start / BUDDY_ORDER0 * BUDDY_ORDER0;
        let start_virt = memory::to_virt(start_aligned as u64);
        let end_aligned = (start + len).next_multiple_of(BUDDY_ORDER0);
        let len_aligned = end_aligned - start_aligned;

        fn inner(
            slab: SlabAllocator,
            start: usize,
            len: usize,
            node: *mut BuddyNode,
            this_order: u32,
            this_address: usize,
        ) {
            unsafe {
                let size = BUDDY_ORDER0 * 2_usize.pow(this_order);
                let end = start + len;
                let this_end = this_address + size;

                //  no overlap at all
                if this_end <= start || this_address >= end {
                    return;
                }

                // node is fully covered
                if this_address >= start && this_end <= end {
                    *node = BuddyNode::Allocated;
                    return;
                }

                // partial overlap
                if this_order == 0 {
                    *node = BuddyNode::Allocated;
                    return;
                }

                let mid = this_address + size / 2;

                if let BuddyNode::Unallocated = *node {
                    let left = slab
                        .allocate_should_recurse(Layout::new::<BuddyNode>(), false)
                        .unwrap()
                        .as_ptr() as *mut BuddyNode;
                    let right = slab
                        .allocate_should_recurse(Layout::new::<BuddyNode>(), false)
                        .unwrap()
                        .as_ptr() as *mut BuddyNode;

                    *left = BuddyNode::Unallocated;
                    *right = BuddyNode::Unallocated;
                    *node = BuddyNode::Branch(left, right);
                }

                if let BuddyNode::Allocated = *node {
                    return;
                }

                if let BuddyNode::Branch(left, right) = *node {
                    inner(
                        slab,
                        start,
                        len,
                        left.as_mut().unwrap(),
                        this_order - 1,
                        this_address,
                    );
                    inner(
                        slab,
                        start,
                        len,
                        right.as_mut().unwrap(),
                        this_order - 1,
                        mid,
                    );
                }
            }
        }

        let (_, start) = memory::ram_start();

        inner(
            data.slab,
            start_virt as usize,
            len_aligned,
            data.root_node,
            data.max_order as u32,
            start as usize,
        );
    }

    fn alloc_should_recurse(&self, layout: Layout, recurse: bool) -> *mut u8 {
        let data = unsafe { self.data.get().as_mut().unwrap().as_mut().unwrap() };

        assert!(data.can_alloc);

        let desired_order = (0..data.max_order)
            .find(|&i| BUDDY_ORDER0 * 2_usize.pow(i as u32) >= layout.size())
            .expect("Allocation too big!") as u32;

        fn inner(
            slab: SlabAllocator,
            node: *mut BuddyNode,
            this_order: u32,
            desired_order: u32,
            this_address: usize,
            recurse: bool,
        ) -> Option<usize> {
            unsafe {
                let this_size = BUDDY_ORDER0 * 2_usize.pow(this_order);
                let mid = this_size / 2;

                // found a suitable place to allocate
                if this_order == desired_order
                    && let BuddyNode::Unallocated = *node
                {
                    *node = BuddyNode::Allocated;
                    return Some(this_address);
                }

                // not a suitable place
                if let BuddyNode::Allocated = *node {
                    return None;
                }
                if this_order <= desired_order {
                    return None;
                }

                // branch and repeat
                if let BuddyNode::Unallocated = *node {
                    let left = slab
                        .allocate_should_recurse(Layout::new::<BuddyNode>(), recurse)
                        .unwrap()
                        .as_ptr() as *mut BuddyNode;
                    let right = slab
                        .allocate_should_recurse(Layout::new::<BuddyNode>(), recurse)
                        .unwrap()
                        .as_ptr() as *mut BuddyNode;

                    *left = BuddyNode::Unallocated;
                    *right = BuddyNode::Unallocated;
                    *node = BuddyNode::Branch(left, right);
                }

                if let BuddyNode::Branch(left, right) = *node {
                    if let Some(addr) = inner(
                        slab,
                        left,
                        this_order - 1,
                        desired_order,
                        this_address,
                        recurse,
                    ) {
                        return Some(addr);
                    } else {
                        return inner(
                            slab,
                            right,
                            this_order - 1,
                            desired_order,
                            this_address + mid,
                            recurse,
                        );
                    }
                }

                None
            }
        }

        let (_, start_addr) = memory::ram_start();

        if let Some(addr) = inner(
            data.slab,
            data.root_node,
            data.max_order as u32,
            desired_order,
            start_addr as usize,
            recurse,
        ) {
            addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc_should_recurse(layout, true)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // TODO: deallocate in buddy allocator
        todo!()
    }
}

#[global_allocator]
static BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    data: UnsafeCell::new(None),
};

#[derive(Clone, Copy)]
struct SlabAllocator(*mut Page);

struct SlabAllocatorFirstHeader {
    next_page: *mut u8,
    step: usize,
    offset: usize,
    first_offset: usize,
    _owned: bool,
    available: usize,

    free_head: *mut SlabAllocatorLink,
}

struct SlabAllocatorHeader {
    next_page: *mut u8,
}

struct SlabAllocatorLink(*mut SlabAllocatorLink);

const REALLOCATE_THRESHOLD: usize = 10;

impl SlabAllocator {
    fn new<T>() -> SlabAllocator {
        unsafe {
            let first_page = BUDDY_ALLOCATOR.alloc_zeroed(Layout::new::<Page>());

            Self::new_no_alloc::<T>(first_page as *mut Page, true)
        }
    }

    fn slots_per_page(first_offset: usize, offset: usize, step: usize) -> (usize, usize) {
        let first = (memory::PAGE_SIZE - first_offset) / step;
        let rest = (memory::PAGE_SIZE - offset) / step;
        (first, rest)
    }

    unsafe fn new_no_alloc<T>(first_page: *mut Page, owned: bool) -> SlabAllocator {
        let align = mem::align_of::<T>().max(mem::align_of::<SlabAllocatorLink>());
        let size = mem::size_of::<T>().max(mem::size_of::<SlabAllocatorLink>());

        let offset = mem::size_of::<SlabAllocatorHeader>().next_multiple_of(align);
        let first_offset = mem::size_of::<SlabAllocatorFirstHeader>().next_multiple_of(align);
        let step = size.next_multiple_of(align);

        let (available, _) = SlabAllocator::slots_per_page(first_offset, offset, step);

        unsafe {
            let first_slot = first_page.byte_add(first_offset) as *mut SlabAllocatorLink;

            for i in 0..available - 1 {
                let link = first_slot.byte_add(step * i);
                let next_link = first_slot.byte_add(step + step * i);

                (*link).0 = next_link;
            }

            let last_link = first_slot.byte_add(step * (available - 1));
            *last_link = SlabAllocatorLink(ptr::null_mut());

            *(first_page as *mut SlabAllocatorFirstHeader) = SlabAllocatorFirstHeader {
                next_page: ptr::null_mut(),
                step,
                offset,
                first_offset,
                _owned: owned,
                available,

                free_head: first_slot,
            };
        }

        SlabAllocator(first_page)
    }

    unsafe fn grow(&self) -> Option<()> {
        unsafe {
            let first = &mut *(self.0 as *mut SlabAllocatorFirstHeader);

            let new_page =
                BUDDY_ALLOCATOR.alloc_should_recurse(Layout::new::<memory::Page>(), false);
            if new_page.is_null() {
                return None;
            }

            *(new_page as *mut SlabAllocatorHeader) = SlabAllocatorHeader {
                next_page: ptr::null_mut(),
            };

            let mut cursor = &mut first.next_page;
            while !(*cursor).is_null() {
                cursor = &mut (*((*cursor) as *mut SlabAllocatorHeader)).next_page;
            }
            *cursor = new_page;

            let first_slot = new_page.byte_add(first.offset) as *mut SlabAllocatorLink;

            let (_, available) = Self::slots_per_page(first.first_offset, first.offset, first.step);

            let last_link = first_slot.byte_add(first.step * (available - 1));
            *last_link = SlabAllocatorLink(ptr::null_mut());

            for i in 0..available - 1 {
                let link = first_slot.byte_add(first.step * i);
                let next_link = first_slot.byte_add(first.step + first.step * i);

                (*link).0 = next_link;
            }

            let mut cursor = &mut (*first.free_head).0;
            while !(cursor.is_null()) {
                cursor = &mut (**cursor).0;
            }

            *cursor = first_slot;

            first.available += available;

            Some(())
        }
    }

    fn allocate_should_recurse(
        &self,
        layout: Layout,
        recurse: bool,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let first = &mut *(self.0 as *mut SlabAllocatorFirstHeader);

            if layout.size() > first.step || layout.align() > first.step {
                return Err(AllocError);
            }

            if first.available < REALLOCATE_THRESHOLD && recurse {
                self.grow();
            }

            if first.available == 0 {
                return Err(AllocError);
            }

            let allocation_ptr = NonNull::new(first.free_head as *mut u8).unwrap();

            let next_free = (*first.free_head).0;
            first.free_head = next_free;

            first.available -= 1;
            let allocation = NonNull::slice_from_raw_parts(allocation_ptr, layout.size());

            Ok(allocation)
        }
    }
}

unsafe impl Allocator for SlabAllocator {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        self.allocate_should_recurse(layout, true)
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: Layout) {
        // TODO: deallocate in slab allocator
        todo!()
    }
}

pub fn setup(boot_info: &BootInfo) {
    BUDDY_ALLOCATOR.init(boot_info);
}
