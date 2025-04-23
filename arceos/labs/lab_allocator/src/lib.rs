//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult};
use axlog::ax_println;
use core::ptr::NonNull;
use core::alloc::Layout;
use allocator::{BuddyByteAllocator, TlsfByteAllocator, SlabByteAllocator};
use memory_addr::{align_down, align_down_4k,align_offset_4k,is_aligned, align_up, align_up_4k, MemoryAddr};

const GREEN: &str = "\u{1B}[32m";
const BLUE: &str = "\u{1B}[34m";
const YELLOW: &str = "\u{1B}[33m";
const RESET: &str = "\u{1B}[0m";

//boot stack: 0xffffffc08020a000 - 0xffffffc08024a000
//boot stack size: 262144
const MAX_POOL_SIZE: usize = 1 << 18;   // 256 KiB 0x40000

const ALLOC_POOL_SIZE: usize = 0x10000; // 64 KiB

//default free memory: 0x8026e000 - 0x88000000
//default free memory size: 0x7d92000
const END_MEMORY: usize = 0x88000000;

pub static mut ODD: bool = true;

/// The size of a 4K page (4096 bytes).
pub const PAGE_SIZE_4K: usize = 0x1000;

pub struct LabByteAllocator{
    pool: TlsfByteAllocator,
    front: usize,
    back: usize,
    alloc_size: usize,
    used_size: usize,
    total_size: usize,
    available_size: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self{
            pool: TlsfByteAllocator::new(),
            front: 0,
            back: 0,
            alloc_size: 0,
            used_size: 0,
            total_size: 0,
            available_size: 0,
        }

    }
}


impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.pool.init(start, MAX_POOL_SIZE);
        self.front = start + MAX_POOL_SIZE;
        self.back = END_MEMORY;
        self.alloc_size = size;
        self.used_size = 0;
        self.total_size = size;
        self.available_size = size;
        ax_println!("{}LabByteAllocator: init, start: {:#x}, size: {:#x}{}",GREEN, start, size, RESET);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!();
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
    let size = layout.size();
    let align = layout.align();
    ax_println!("{}LabByteAllocator: alloc, size: {:#x}, align: {:#x}{}", GREEN, size, align, RESET);
        if layout.align() == 8{
            let result = self.pool.alloc(layout);
            match result {
                Ok(ptr) => {
                    self.alloc_size += layout.size();
                    self.used_size += align_up(layout.size(), layout.align()) ;
                    self.available_size -= align_up(layout.size(), layout.align()) ;
                    unsafe {ODD = !ODD;}
                    ax_println!("{}LabByteAllocator: alloc, ptr: {:#x}, size: {:#x}{}", GREEN, ptr.as_ptr() as usize, layout.size(), RESET);
                    Ok(ptr)
                }
                Err(_) => {
                    ax_println!("{}LabByteAllocator: alloc failed{}", YELLOW, RESET);
                    return Err(allocator::AllocError::NoMemory)
                }
            }
        }else {
            if unsafe {ODD} 
            {
                unsafe {ODD = !ODD;}
                let aligned_pos = align_up(self.front, align);
                let new_pos = aligned_pos.checked_add(size).ok_or(allocator::AllocError::MemoryOverlap)?;
                if new_pos > self.back {
                    return Err(allocator::AllocError::MemoryOverlap);
                }
                self.alloc_size += layout.size();
                self.used_size += align_up(layout.size(), layout.align());
                self.available_size -= align_up(layout.size(), layout.align());
                NonNull::new(aligned_pos as *mut u8)
            .ok_or(allocator::AllocError::InvalidParam)
            }else {
                unsafe {ODD = !ODD;}
                let aligned_pos = align_down(self.back, align);
                let new_pos = aligned_pos.checked_sub(size).ok_or(allocator::AllocError::MemoryOverlap)?;
                if new_pos < self.front {
                    return Err(allocator::AllocError::MemoryOverlap);
                }
                self.alloc_size += layout.size();
                self.used_size += align_up(layout.size(), layout.align());
                self.available_size -= align_up(layout.size(), layout.align());
                NonNull::new(aligned_pos as *mut u8)
            .ok_or(allocator::AllocError::InvalidParam)
            }
            
        }

    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        if layout.size() <= ALLOC_POOL_SIZE{
            self.pool.dealloc(pos, layout);
            self.alloc_size -= layout.size();
            self.used_size -= align_up(layout.size(), layout.align());
            self.available_size += align_up(layout.size(), layout.align());
            ax_println!("{}LabByteAllocator: dealloc, ptr: {:#x}, size: {:#x}{}", GREEN, pos.as_ptr() as usize, layout.size(), RESET);
        }else {
            if unsafe {ODD} {
                self.alloc_size -= layout.size();
                self.used_size -= align_up(layout.size(), layout.align());
                self.available_size += align_up(layout.size(), layout.align());
            }else {
                self.alloc_size -= layout.size();
                self.used_size -= align_up(layout.size(), layout.align());
                self.available_size += align_up(layout.size(), layout.align());
            }
        }
    }
    fn total_bytes(&self) -> usize {
        self.total_size
    }
    fn used_bytes(&self) -> usize {
        self.used_size
    }
    fn available_bytes(&self) -> usize {
        self.available_size
    }
}
