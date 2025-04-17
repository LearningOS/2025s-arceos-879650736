#![no_std]

use core::alloc::Layout;
use core::ptr::NonNull;

use allocator::{AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

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
pub struct EarlyAllocator {
    // Memory region boundaries
    start: usize,
    end: usize,
    
    // Current positions for both allocators
    bytes_pos: usize,  // Position for next byte allocation
    pages_pos: usize,  // Position for next page allocation
    
    // Size of the bytes area
    byte_size: usize,
    // Size of the pages area
    page_size: usize,

    // Tracking allocation counts for bytes area
    bytes_count: usize,
}

impl EarlyAllocator {
    // Create a new uninitialized allocator
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            bytes_pos: 0,
            byte_size: 0,
            pages_pos: 0,
            page_size: 0,
            bytes_count: 0,
        }
    }
    
    // Check if memory range is valid
    fn is_valid_range(&self, start: usize, size: usize) -> bool {
        // Check for potential overflow
        if start.checked_add(size).is_none() {
            return false;
        }
        // Size must be non-zero
        size > 0
    }
    
    // Align address upward
    fn align_up(&self, addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
}

impl BaseAllocator for EarlyAllocator {
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize) {
        if self.is_valid_range(start, size) {
            self.start = start;
            self.end = start + size;
            self.bytes_pos = start;
            self.pages_pos = self.end;
            self.byte_size += size;
            self.page_size += size;
            self.bytes_count = 0;
        }
    }

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // Early allocator can only manage a single contiguous region
        // Additional regions cannot be added
        unimplemented!()
    }
}

impl ByteAllocator for EarlyAllocator {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        
        if size == 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        
        // Align the current byte position upward
        let aligned_pos = self.align_up(self.bytes_pos, align);
        
        // Check if we have enough space
        let new_pos = aligned_pos.checked_add(size).ok_or(allocator::AllocError::MemoryOverlap)?;
        
        // Ensure we don't overlap with the page allocator area
        if new_pos > self.pages_pos {
            return Err(allocator::AllocError::MemoryOverlap);
        }
        
        // Update the bytes position and allocation count
        let allocation_start = aligned_pos;
        self.bytes_pos = new_pos;
        self.bytes_count += 1;
        self.byte_size += size;
        
        // Convert to NonNull pointer
        NonNull::new(allocation_start as *mut u8)
            .ok_or(allocator::AllocError::InvalidParam)
    }

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        // We can't free individual allocations in the middle
        // We only track the count of allocations
        if self.bytes_count > 0 {
            self.bytes_count -= 1;
            
            // If we've deallocated all bytes, reset the byte position
            if self.bytes_count == 0 {
                self.bytes_pos = self.start;
                self.byte_size = 0;
            }
        }
    }

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize {
        self.bytes_pos - self.start
    }

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize {
        // Bytes area + Pages area
        self.byte_size
    }

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize {
        if self.pages_pos > self.bytes_pos {
            self.pages_pos - self.bytes_pos
        } else {
            0
        }
    }
}

impl PageAllocator for EarlyAllocator {
    const PAGE_SIZE: usize = 4096; // Typical page size, adjust as needed

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if num_pages == 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        
        let align = 1 << align_pow2;
        let bytes_needed = num_pages * Self::PAGE_SIZE;
        
        // Calculate aligned position (aligning downward for page allocation)
        let aligned_pos = (self.pages_pos - bytes_needed) & !(align - 1);
        
        // Check if we have enough space
        if aligned_pos < self.bytes_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        
        self.page_size += bytes_needed;
        // Update the page position
        self.pages_pos = aligned_pos;
        
        Ok(aligned_pos)
    }

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are never deallocated in this allocator
        // So this is a no-op according to the requirements
    }

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize {
        self.pages_pos / Self::PAGE_SIZE
    }

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize {
        self.page_size / Self::PAGE_SIZE
    }

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize {
        if self.pages_pos > self.bytes_pos {
            (self.pages_pos - self.bytes_pos) / Self::PAGE_SIZE
        } else {
            0
        }
    }
}