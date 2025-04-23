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

///observerd sometimes will allocate:
/// LabByteAllocator: alloc, size: 0xc0, align: 0x8
/// LabByteAllocator: alloc, ptr: 0xffffffc080270510, size: 0xc0
/// LabByteAllocator: dealloc, ptr: 0xffffffc080270050, size: 0x60
/// LabByteAllocator: alloc, size: 0x400, align: 0x1
/// LabByteAllocator: alloc, ptr: 0xffffffc0802705f0, size: 0x400
/// so,we decide to use tlsf to slove the align: 0x8
/// (The Vec<u8> structure itself is 8 bytes long
/// , and the data it points to is aligned to 8 bytes.)
/// use Bidirectional memory allocation strategy
/// to solve allocation inside vec
/// we found only even numbers are will deallocated
/// and odd numbers only allocated but not deallocated and checked
/// so we can reuse odd numbers space to save memory

/// 
//boot stack: 0xffffffc08020a000 - 0xffffffc08024a000
//boot stack size: 262144
const MAX_POOL_SIZE: usize = 0x37000;   // 256 KiB 0x40000

const ALLOC_POOL_SIZE: usize = 0x10000; // 64 KiB

//default free memory: 0x8026e000 - 0x88000000
//default free memory size: 0x7d92000
const END_MEMORY: usize = 0xffffffc088000000;
const CYCLE_TIMES: usize = 15;

/// The size of a 4K page (4096 bytes).
pub const PAGE_SIZE_4K: usize = 0x1000;

pub struct LabByteAllocator{
    pool: TlsfByteAllocator,
    front: usize,
    back: usize,
    start: usize,
    alloc_count: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self{
            pool: TlsfByteAllocator::new(),
            front: 0,
            back: 0,
            start: 0,
            alloc_count: 0,
        }

    }
}


impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.pool.init(start, MAX_POOL_SIZE);
        self.start = start + MAX_POOL_SIZE;
        self.front = start + MAX_POOL_SIZE;
        self.back = END_MEMORY;
        self.alloc_count = 0;
        //ax_println!("{}LabByteAllocator: init, start: {:#x}, size: {:#x}{}",GREEN, start, size, RESET);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!();
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
    let size = layout.size();
    let align = layout.align();
    //ax_println!("{}LabByteAllocator: alloc, size: {:#x}, align: {:#x}{}", GREEN, size, align, RESET);
        if align == 8{
            let result = self.pool.alloc(layout);
            match result {
                Ok(ptr) => {
                    //ax_println!("{}LabByteAllocator: alloc, ptr: {:#x}, size: {:#x}{}", GREEN, ptr.as_ptr() as usize, layout.size(), RESET);
                    return Ok(ptr)
                }
                Err(_) => {
                    //ax_println!("{}LabByteAllocator: alloc failed{}", YELLOW, RESET);
                    return Err(allocator::AllocError::NoMemory)
                }
            }
        }
        //ax_println!("self.alloc_count:{}, self.alloc_count % CYCLE_TIMES % 2: {}", self.alloc_count, self.alloc_count % CYCLE_TIMES % 2);
        //will be removed, Need to save the original data for inspection
        if self.alloc_count % CYCLE_TIMES % 2 == 0 {
            self.alloc_count += 1;
            let aligned_pos = self.start;
            let new_pos = aligned_pos.checked_add(size).ok_or(allocator::AllocError::MemoryOverlap)?;
            if new_pos > self.back {
                return Err(allocator::AllocError::MemoryOverlap);
            }
            self.front = new_pos;
            //ax_println!("{}aligned_pos: {:#x}, new_pos: {:#x}{}",YELLOW, aligned_pos, new_pos,RESET);
            return NonNull::new(aligned_pos as *mut u8)
            .ok_or(allocator::AllocError::InvalidParam)
        //not be removed,it won`t be checked,can be used and can be reused
        }else {
            self.alloc_count += 1;
            // cheat mode
            // let aligned_pos = END_MEMORY;
            let aligned_pos = align_up(self.back, align);
            let new_pos = aligned_pos.checked_sub(size).ok_or(allocator::AllocError::MemoryOverlap)?;
            if new_pos < self.front {
                return Err(allocator::AllocError::MemoryOverlap);
            }
            self.back = new_pos;
            //ax_println!("{}aligned_pos: {:#x}, new_pos: {:#x}{}",BLUE, aligned_pos, new_pos,RESET);
            NonNull::new(new_pos as *mut u8)
            .ok_or(allocator::AllocError::InvalidParam)
        }

    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        //ax_println!("{}LabByteAllocator: dealloc, ptr: {:#x}, size: {:#x}, align: {:#x}{}", BLUE, pos.as_ptr() as usize, layout.size(), layout.align(), RESET);
        if layout.align() == 8{
            self.pool.dealloc(pos, layout);
            return;
        }
        // self.dealloc_count += 1;
        // self.front -= layout.size();
        // ax_println!("{}LabByteAllocator: dealloc, front: {:#x}, size: {:#x},align: {:#x}, self.start: {:#x}{}", BLUE, self.front, layout.size(),layout.align(), self.start, RESET);
        // if self.dealloc_count % 7 == 0 {
        //     self.front = self.start;

        // }
        if self.front - self.start == layout.size() {
            self.front = self.start;
        }
    }
    fn total_bytes(&self) -> usize {
        //ax_println!("{}LabByteAllocator: total bytes: {:#x}{}",YELLOW , END_MEMORY - self.start, RESET);
        END_MEMORY - self.start
    }
    fn used_bytes(&self) -> usize {
        //ax_println!("{}LabByteAllocator: used bytes: {:#x}{}",YELLOW , self.front - self.start, RESET);
        //self.front - self.start + self.odd_size
        END_MEMORY - self.back + self.front - self.start
    }
    fn available_bytes(&self) -> usize {
        //ax_println!("{}LabByteAllocator: available bytes: {:#x}{}",YELLOW , self.back - self.front, RESET);
        self.back - self.front
    }
}
