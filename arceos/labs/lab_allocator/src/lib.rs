//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult};
use axlog::ax_println;
use core::ptr::NonNull;
use core::alloc::Layout;
use allocator::{BuddyByteAllocator, TlsfByteAllocator, SlabByteAllocator};

const GREEN: &str = "\u{1B}[32m";
const BLUE: &str = "\u{1B}[34m";
const YELLOW: &str = "\u{1B}[33m";
const RESET: &str = "\u{1B}[0m";

//boot stack: 0xffffffc08020a000 - 0xffffffc08024a000
//boot stack size: 262144
const MAX_POOL_SIZE: usize = 1 << 18;   // 256 KiB

//default free memory: 0x8026e000 - 0x88000000
//default free memory size: 0x7d92000
const END_MEMORY: usize = 0x88000000;


pub struct LabByteAllocator{
    pool: TlsfByteAllocator,
    front: usize,
    back: usize,
    size: usize,
    used: usize,
    available: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self{
            pool: TlsfByteAllocator::new(),
            front: 0,
            back: 0,
            size: 0,
            used: 0,
            available: 0,
        }

    }
}


impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.pool.init(start, MAX_POOL_SIZE);
        self.front = start + MAX_POOL_SIZE;
        self.back = END_MEMORY;
        self.size = size;
        self.used = 0;
        self.available = size;
        ax_println!("{}LabByteAllocator: init, start: {:#x}, size: {:#x}{}",GREEN, start, size, RESET);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!();
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        unimplemented!();
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        unimplemented!();
    }
    fn total_bytes(&self) -> usize {
        unimplemented!();
    }
    fn used_bytes(&self) -> usize {
        unimplemented!();
    }
    fn available_bytes(&self) -> usize {
        unimplemented!();
    }
}
