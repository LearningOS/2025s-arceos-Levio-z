#![no_std]
#![no_main]
extern crate alloc;
use allocator::{BaseAllocator, ByteAllocator, PageAllocator,AllocResult,AllocError};
use core::ptr::NonNull;
use core::alloc::{ Layout};
/// 早期内存分配器
/// 在正式的字节分配器和页分配器可用之前使用它！
/// 这是一个双端内存范围：
/// - 向前分配字节
/// - 向后分配页
///
/// [ 已用字节区 | 可用区域 | 已用页区 ]
/// |           | -->  <-- |          |
/// start      b_pos     p_pos      end
///
/// 对于字节区域，'count'记录分配次数。
/// 当它降为零时，释放已用字节区。
/// 对于页区域，它永远不会被释放！
///
/// The error type used for allocation.

/// A [`Result`] type with [`AllocError`] as the error type.
pub struct EarlyAllocator<const PAGE_SIZE: usize>{
    start:  usize,
    end:    usize,
    count:  usize,
    byte_pos: usize,
    page_pos: usize,
}

impl <const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        return Self::uninit_new()
    }
    pub fn new_with_init(&mut self, start: usize, size: usize)  -> Self{
        let mut init = Self::uninit_new();
        init.init(start, size);
        init
    }
    pub fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.byte_pos = start;
        self.page_pos = self.end;
    }
    pub const fn uninit_new() -> Self {
        Self {
            start: 0, end: 0, count: 0,
            byte_pos: 0, page_pos: 0,
        }
    }
}

impl <const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE>{
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize){
        self.new_with_init(start, size);
    }

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult{
        unimplemented!()
    }
}

impl <const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator <PAGE_SIZE>{
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let start = align_up(self.byte_pos, layout.align());
        let next = start + layout.size();
        if next > self.page_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.byte_pos = next;
            self.count += 1;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.byte_pos = self.start;
        }
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

impl <const PAGE_SIZE: usize> PageAllocator for EarlyAllocator <PAGE_SIZE>{
    const PAGE_SIZE: usize = 0x1000;
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let layout: Layout = Layout::from_size_align(num_pages*PAGE_SIZE,align_pow2).map_err(|_|AllocError::InvalidParam)?;
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let next = align_down(self.page_pos - layout.size(), layout.align());
        if next <= self.byte_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.page_pos = next;
            Ok(next)
        }
    }
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize){
        unimplemented!()
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

#[inline]
const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

#[inline]
const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}