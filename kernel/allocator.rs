use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x_4444_4444_0000; // virtual adress for the heap
pub const HEAP_SIZE: usize = 256 * 1024; //256 kb heap

// reserve a static array in the kernel image
static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init() {
    unsafe {
        let start = &raw mut HEAP_SPACE as *mut u8;
        ALLOCATOR.lock().init(start, HEAP_SIZE);
    }
}

