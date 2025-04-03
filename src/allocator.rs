use buddy_system_allocator::LockedHeap;
unsafe extern "C" {
    fn vdso_data();
}

const PAGE_SIZE: usize = 0x1000;

static COPS_HEAP_SIZE: usize = 8 * PAGE_SIZE;

pub type Allocator = LockedHeap<32>;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub fn init() {
    let start = vdso_data as usize;
    let end = start + COPS_HEAP_SIZE;
    unsafe {
        ALLOCATOR.lock().init(start, end);
    }
}
