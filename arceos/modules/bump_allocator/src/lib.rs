#![no_std]

use core::{alloc::Layout, ptr::NonNull};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    byte_pos: usize,
    page_pos: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            byte_pos: 0,
            page_pos: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) { 
        self.start = start;
        self.end = start + size;
        self.byte_pos = start;
        self.page_pos = self.end;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!() // unsupported
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();

        let aligned_start = (self.byte_pos + align - 1) & !(align - 1);
        if aligned_start + size > self.page_pos {
            return Err(AllocError::NoMemory);
        }

        self.byte_pos = aligned_start + size;

        Ok(unsafe { NonNull::new_unchecked(aligned_start as *mut u8) })
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        // unsupported
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        
        let size = num_pages * PAGE_SIZE;
        let align = 1 << align_pow2;

        let new_page_pos = (self.page_pos - size) & !(align - 1);
        if new_page_pos < self.byte_pos {
            return Err(AllocError::NoMemory);
        }

        self.page_pos = new_page_pos;
        Ok(new_page_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // unsupported
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}
