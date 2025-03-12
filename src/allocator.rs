use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

// use spin::Mutex;
use spinlock::SpinRaw;

/// 共享代码中默认的分配器，使用的是内核和用户程序各自的堆
/// 前提：堆的虚拟地址都保存在 HEAP_BUFFER 这个虚拟地址中
/// 分配和回收时，先读取 HEAP_BUFFER 虚拟地址中的内容
/// 再类型转换成正确的数据结构指针
/// 如果是把 heap 的指针当作参数传进需要使用的代码中，那么在分配的时候，需要显式的指出堆分配器
/// 通过这种方式，可以让默认的分配器使用不同的堆
#[global_allocator]
static GLOBAL: Global = Global;

// const PAGE_SIZE: usize = 0x1000;
// #[repr(align(1024))]
// struct HeapData([u8; PAGE_SIZE]);

// // #[link_section = "data"]
// static HEAP_DATA: HeapData = HeapData([0; PAGE_SIZE]);

use buddy_system_allocator::Heap;

use crate::get_alloc_area;
type LockedHeap = SpinRaw<Heap>;
// type LockedHeap = Mutex<Heap>;

struct Global;
unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heapptr = get_alloc_area().0.as_ptr();
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).lock().alloc(layout).ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heapptr = get_alloc_area().0.as_ptr();
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}