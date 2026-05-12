use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    mem,
    ptr::{self, NonNull},
};

use alloc::alloc::AllocError;

use crate::{
    arch::boot::BootInfo,
    memory::{self, Page, ZEROED_PAGE},
    sync::{SpinLock, SpinLockGuard},
};

struct BuddyAllocator {
    inner: SpinLock<Option<BuddyInner>>,
}

struct BuddyHeader(*mut Page);

struct BuddyLink(*mut BuddyLink);

struct BuddyInner {
    root_node: *mut BuddyNode,
    max_order: usize,
    can_alloc: bool,

    free_head: *mut BuddyLink,
    first_page: *mut Page,
    capacity: usize,
    needs_grow: bool,

    free_ram: usize,
}

impl BuddyInner {
    #[inline(always)]
    fn align() -> usize {
        mem::align_of::<BuddyNode>().next_multiple_of(mem::align_of::<BuddyLink>())
    }

    #[inline(always)]
    fn offset() -> usize {
        let align = Self::align();
        mem::size_of::<BuddyHeader>().next_multiple_of(align)
    }

    #[inline(always)]
    fn size() -> usize {
        mem::size_of::<BuddyNode>().max(mem::size_of::<BuddyLink>())
    }

    #[inline(always)]
    fn step() -> usize {
        let align = Self::align();
        Self::size().next_multiple_of(align)
    }

    #[inline(always)]
    fn count() -> usize {
        let offset = Self::offset();
        let step = Self::step();
        (memory::PAGE_SIZE - offset) / step
    }

    unsafe fn init_page(page: *mut Page) {
        unsafe {
            *(page as *mut BuddyHeader) = BuddyHeader(ptr::null_mut());

            let step = Self::step();
            let count = Self::count();
            let first_slot = page.byte_add(Self::offset()) as *mut BuddyLink;

            for i in 0..count - 1 {
                let slot = first_slot.byte_add(i * step);
                let next = slot.byte_add(Self::step());

                *slot = BuddyLink(next);
            }

            let last_slot = first_slot.byte_add((count - 1) * step);
            *last_slot = BuddyLink(ptr::null_mut());
        }
    }

    fn new(boot_info: &BootInfo) -> BuddyInner {
        unsafe {
            let max_order = (0..u32::MAX)
                .find(|&i| {
                    BUDDY_ORDER0 * 2_usize.pow(i) > boot_info.memory_info.memory.len as usize
                })
                .unwrap();

            let first_page = &raw mut BUDDY_PAGE;

            Self::init_page(first_page);

            let offset = Self::offset();
            let step = Self::step();

            let root_node = first_page.byte_add(offset) as *mut BuddyNode;
            *root_node = BuddyNode::Unallocated;

            let free_head = first_page.byte_add(offset + step) as *mut BuddyLink;

            let mut inner = BuddyInner {
                root_node,
                max_order: max_order as usize,
                can_alloc: false,

                first_page,
                free_head,
                capacity: Self::count() - 1,
                needs_grow: false,

                free_ram: 2_usize.pow(max_order) * BUDDY_ORDER0,
            };

            for memory::MemoryRange { start, len } in &boot_info.resv[0..boot_info.resv_count] {
                inner.reserve_range(*start as usize, *len as usize);
            }

            let allocation_end =
                boot_info.memory_info.memory.phys as usize + BUDDY_ORDER0 * 2_usize.pow(max_order);
            let memory_end =
                (boot_info.memory_info.memory.phys + boot_info.memory_info.memory.len) as usize;
            let len = allocation_end - memory_end;

            inner.reserve_range(memory_end, len);

            inner.can_alloc = true;

            inner
        }
    }

    fn grow(&mut self) {
        unsafe {
            log::debug!("buddy allocator: growing!");

            self.needs_grow = false;

            let new_page = self.alloc(Layout::new::<memory::Page>()) as *mut Page;
            assert!(!new_page.is_null());

            Self::init_page(new_page);

            *(new_page as *mut BuddyHeader) = BuddyHeader(self.first_page);

            let offset = Self::offset();
            let step = Self::step();
            let count = Self::count();

            let first_link = new_page.byte_add(offset) as *mut BuddyLink;
            let last_link = first_link.byte_add(step * (count - 1));

            *last_link = BuddyLink(self.free_head);
            self.free_head = first_link;

            self.first_page = new_page;

            self.capacity += count;
        }
    }

    fn new_node(&mut self) -> Option<*mut BuddyNode> {
        let reallocate_threshold = 2 * self.max_order;

        if self.capacity == 0 {
            log::error!("buddy allocator: out of node capacity");

            return None;
        }

        let free_head = unsafe { (*self.free_head).0 };
        let new_node = self.free_head as *mut BuddyNode;
        unsafe { *new_node = Default::default() };
        self.free_head = free_head;

        self.capacity -= 1;

        if self.capacity <= reallocate_threshold {
            self.needs_grow = true;
        }

        Some(new_node)
    }

    fn free_node(&mut self, node: *mut BuddyNode) {
        unsafe {
            let link = node as *mut BuddyLink;
            *link = BuddyLink(self.free_head);

            self.free_head = link;
            self.capacity += 1;
        }
    }

    fn reserve_range(&mut self, start: usize, len: usize) {
        let (addr_start, _) = memory::ram_start();

        let start_aligned = start / BUDDY_ORDER0 * BUDDY_ORDER0;
        let end_aligned = (start + len).next_multiple_of(BUDDY_ORDER0);
        let len_aligned = end_aligned - start_aligned;

        fn inner(
            this: &mut BuddyInner,
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
                    this.free_ram -= size;
                    return;
                }

                // partial overlap
                if this_order == 0 {
                    *node = BuddyNode::Allocated;
                    this.free_ram -= size;
                    return;
                }

                let mid = this_address + size / 2;

                if let BuddyNode::Unallocated = *node {
                    let left = this.new_node().unwrap();
                    let right = this.new_node().unwrap();

                    *left = BuddyNode::Unallocated;
                    *right = BuddyNode::Unallocated;
                    *node = BuddyNode::Branch(left, right);
                }

                if let BuddyNode::Allocated = *node {
                    return;
                }

                if let BuddyNode::Branch(left, right) = *node {
                    inner(
                        this,
                        start,
                        len,
                        left.as_mut().unwrap(),
                        this_order - 1,
                        this_address,
                    );
                    inner(
                        this,
                        start,
                        len,
                        right.as_mut().unwrap(),
                        this_order - 1,
                        mid,
                    );
                }
            }
        }

        inner(
            self,
            start_aligned,
            len_aligned,
            self.root_node,
            self.max_order as u32,
            addr_start as usize,
        );
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        assert!(self.can_alloc);

        if self.needs_grow {
            self.grow();
        }

        let desired_order = (0..=self.max_order)
            .find(|&i| BUDDY_ORDER0 * 2_usize.pow(i as u32) >= layout.size())
            .expect("Allocation too big!") as u32;

        fn inner(
            this: &mut BuddyInner,
            node: *mut BuddyNode,
            this_order: u32,
            desired_order: u32,
            this_address: usize,
        ) -> Option<usize> {
            unsafe {
                let this_size = BUDDY_ORDER0 * 2_usize.pow(this_order);
                let mid = this_address + this_size / 2;

                // found a suitable place to allocate
                if this_order == desired_order
                    && let BuddyNode::Unallocated = *node
                {
                    *node = BuddyNode::Allocated;
                    this.free_ram -= this_size;
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
                    let left = this.new_node().unwrap();
                    let right = this.new_node().unwrap();

                    *left = BuddyNode::Unallocated;
                    *right = BuddyNode::Unallocated;
                    *node = BuddyNode::Branch(left, right);
                }

                if let BuddyNode::Branch(left, right) = *node {
                    if let Some(addr) =
                        inner(this, left, this_order - 1, desired_order, this_address)
                    {
                        return Some(addr);
                    } else {
                        return inner(this, right, this_order - 1, desired_order, mid);
                    }
                }

                None
            }
        }

        let (_, start_addr) = memory::ram_start();

        if let Some(addr) = inner(
            self,
            self.root_node,
            self.max_order as u32,
            desired_order,
            start_addr as usize,
        ) {
            addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    fn dealloc(&mut self, ptr: *mut u8) {
        fn inner(
            this: &mut BuddyInner,
            addr: usize,
            this_node: *mut BuddyNode,
            this_order: u32,
            this_addr: usize,
        ) {
            unsafe {
                let this_size = 2_usize.pow(this_order) * BUDDY_ORDER0;
                let mid = this_addr + this_size / 2;

                if let BuddyNode::Branch(left, right) = *this_node {
                    if addr >= mid {
                        inner(this, addr, right, this_order - 1, mid);
                    } else {
                        inner(this, addr, left, this_order - 1, this_addr);
                    }

                    if let BuddyNode::Unallocated = *left
                        && let BuddyNode::Unallocated = *right
                    {
                        this.free_node(left);
                        this.free_node(right);

                        *this_node = BuddyNode::Unallocated;
                    }

                    return;
                }

                if let BuddyNode::Allocated = *this_node {
                    *this_node = BuddyNode::Unallocated;
                    this.free_ram += this_size;

                    return;
                }

                panic!("invalid buddy state");
            }
        }

        let (_, start_addr) = memory::ram_start();

        let addr = ptr as usize;
        inner(
            self,
            addr,
            self.root_node,
            self.max_order as u32,
            start_addr as usize,
        );
    }
}

unsafe impl Sync for BuddyAllocator {}

#[derive(Default)]
enum BuddyNode {
    #[default]
    Unallocated,
    Allocated,
    Branch(*mut BuddyNode, *mut BuddyNode),
}

static mut BUDDY_PAGE: Page = ZEROED_PAGE;

const BUDDY_ORDER0: usize = memory::PAGE_SIZE;

impl BuddyAllocator {
    fn init(&self, boot_info: &BootInfo) {
        let mut lock = self.inner.lock();
        assert!(lock.is_none());

        let inner = BuddyInner::new(boot_info);
        *lock = Some(inner);
    }

    fn free_ram(&self) -> memory::Size {
        memory::Size(self.inner.lock().as_ref().unwrap().free_ram)
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.inner.lock().as_mut().unwrap().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        self.inner.lock().as_mut().unwrap().dealloc(ptr)
    }
}

static BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    inner: SpinLock::new(None),
};

#[derive(Clone, Copy)]
pub struct SlabAllocator(*mut Page);

struct SlabAllocatorInner {
    next_page: *mut u8,
    step: usize,
    offset: usize,
    first_offset: usize,
    available: usize,
    name: &'static str,

    free_head: *mut SlabAllocatorLink,
}

struct SlabAllocatorFirstHeader(SpinLock<SlabAllocatorInner>);

struct SlabAllocatorHeader {
    next_page: *mut u8,
}

struct SlabAllocatorLink(*mut SlabAllocatorLink);

const REALLOCATE_THRESHOLD: usize = 32;

impl SlabAllocator {
    fn slots_per_page(first_offset: usize, offset: usize, step: usize) -> (usize, usize) {
        let first = (memory::PAGE_SIZE - first_offset) / step;
        let rest = (memory::PAGE_SIZE - offset) / step;
        (first, rest)
    }

    pub fn new<T>(name: &'static str) -> SlabAllocator {
        let first_page =
            unsafe { BUDDY_ALLOCATOR.alloc_zeroed(Layout::new::<Page>()) as *mut Page };

        assert!(!first_page.is_null());

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

            let inner = SlabAllocatorInner {
                next_page: ptr::null_mut(),
                step,
                offset,
                first_offset,
                available,
                name,

                free_head: first_slot,
            };

            *(first_page as *mut SlabAllocatorFirstHeader) =
                SlabAllocatorFirstHeader(SpinLock::new(inner));
        }

        SlabAllocator(first_page)
    }

    unsafe fn grow(&self, lock: &mut SpinLockGuard<SlabAllocatorInner>) -> Option<()> {
        unsafe {
            log::debug!("{}: growing!", lock.name);

            let new_page = BUDDY_ALLOCATOR.alloc(Layout::new::<memory::Page>());
            if new_page.is_null() {
                return None;
            }

            *(new_page as *mut SlabAllocatorHeader) = SlabAllocatorHeader {
                next_page: ptr::null_mut(),
            };

            let mut cursor = &mut lock.next_page;
            while !(*cursor).is_null() {
                let ptr = (*cursor) as *mut SlabAllocatorHeader;
                cursor = &mut (*ptr).next_page;
            }
            *cursor = new_page;

            let first_slot = new_page.byte_add(lock.offset) as *mut SlabAllocatorLink;

            let (_, available) = Self::slots_per_page(lock.first_offset, lock.offset, lock.step);

            let last_link = first_slot.byte_add(lock.step * (available - 1));
            *last_link = SlabAllocatorLink(lock.free_head);

            for i in 0..available - 1 {
                let link = first_slot.byte_add(lock.step * i);
                let next_link = first_slot.byte_add(lock.step + lock.step * i);

                (*link).0 = next_link;
            }

            lock.free_head = first_slot;

            lock.available += available;

            Some(())
        }
    }
}

unsafe impl Allocator for SlabAllocator {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        unsafe {
            let first = &*(self.0 as *mut SlabAllocatorFirstHeader);
            let mut lock = first.0.lock();

            if layout.size() > lock.step || layout.align() > lock.step {
                log::error!("{}: bad allocation layout", lock.name);

                return Err(AllocError);
            }

            if lock.available < REALLOCATE_THRESHOLD && self.grow(&mut lock).is_none() {
                log::error!("{}: failed to grow slab", lock.name);

                return Err(AllocError);
            }

            if lock.available == 0 {
                log::error!("{}: slab out of capacity", lock.name);

                return Err(AllocError);
            }

            let allocation_ptr = NonNull::new(lock.free_head as *mut u8).unwrap();

            let next_free = (*lock.free_head).0;
            lock.free_head = next_free;

            lock.available -= 1;
            let allocation = NonNull::slice_from_raw_parts(allocation_ptr, layout.size());

            Ok(allocation)
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, _layout: Layout) {
        unsafe {
            let first = &*(self.0 as *mut SlabAllocatorFirstHeader);
            let mut lock = first.0.lock();

            let link = ptr.as_ptr() as *mut SlabAllocatorLink;
            *link = SlabAllocatorLink(lock.free_head);

            lock.free_head = link;
            lock.available += 1;
        }
    }
}

struct AutoAllocatorInner {
    size_8: SlabAllocator,
    size_16: SlabAllocator,
    size_32: SlabAllocator,
    size_64: SlabAllocator,
    size_128: SlabAllocator,
    size_256: SlabAllocator,
    size_512: SlabAllocator,
    size_1k: SlabAllocator,
}

impl AutoAllocatorInner {
    fn new() -> AutoAllocatorInner {
        AutoAllocatorInner {
            size_8: SlabAllocator::new::<[u64; 1]>("auto_size_8"),
            size_16: SlabAllocator::new::<[u64; 2]>("auto_size_16"),
            size_32: SlabAllocator::new::<[u64; 4]>("auto_size_32"),
            size_64: SlabAllocator::new::<[u64; 8]>("auto_size_64"),
            size_128: SlabAllocator::new::<[u64; 16]>("auto_size_128"),
            size_256: SlabAllocator::new::<[u64; 32]>("auto_size_256"),
            size_512: SlabAllocator::new::<[u64; 64]>("auto_size_512"),
            size_1k: SlabAllocator::new::<[u64; 128]>("auto_size_1k"),
        }
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            if layout.align() > 8 || layout.size() > 1024 {
                return BUDDY_ALLOCATOR.alloc(layout);
            }

            let allocator = if layout.size() <= 8 {
                self.size_8
            } else if layout.size() <= 16 {
                self.size_16
            } else if layout.size() <= 32 {
                self.size_32
            } else if layout.size() <= 64 {
                self.size_64
            } else if layout.size() <= 128 {
                self.size_128
            } else if layout.size() <= 256 {
                self.size_256
            } else if layout.size() <= 512 {
                self.size_512
            } else {
                self.size_1k
            };

            allocator
                .allocate(layout)
                .map(|mut val| val.as_mut().as_mut_ptr())
                .unwrap_or(ptr::null_mut())
        }
    }

    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe {
            if layout.align() > 8 || layout.size() > 1024 {
                BUDDY_ALLOCATOR.dealloc(ptr, layout);
                return;
            }

            let allocator = if layout.size() <= 8 {
                self.size_8
            } else if layout.size() <= 16 {
                self.size_16
            } else if layout.size() <= 32 {
                self.size_32
            } else if layout.size() <= 64 {
                self.size_64
            } else if layout.size() <= 128 {
                self.size_128
            } else if layout.size() <= 256 {
                self.size_256
            } else if layout.size() <= 512 {
                self.size_512
            } else {
                self.size_1k
            };

            allocator.deallocate(NonNull::new(ptr).unwrap(), layout);
        }
    }
}

struct AutoAllocator {
    inner: SpinLock<Option<AutoAllocatorInner>>,
}

impl AutoAllocator {
    fn init(&self) {
        let mut lock = self.inner.lock();
        assert!(lock.is_none());

        let inner = AutoAllocatorInner::new();

        *lock = Some(inner);
    }
}

unsafe impl GlobalAlloc for AutoAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().as_mut().unwrap().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().as_mut().unwrap().dealloc(ptr, layout)
    }
}

unsafe impl Sync for AutoAllocator {}

#[global_allocator]
static AUTO_ALLOCATOR: AutoAllocator = AutoAllocator {
    inner: SpinLock::new(None),
};

pub fn free_ram() -> memory::Size {
    BUDDY_ALLOCATOR.free_ram()
}

pub fn setup(boot_info: &BootInfo) {
    BUDDY_ALLOCATOR.init(boot_info);
    AUTO_ALLOCATOR.init();
}
