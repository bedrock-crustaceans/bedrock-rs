use bedrock_protocol_core::ProtoCodec;
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};

struct LargestAllocation;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for LargestAllocation {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LARGEST.fetch_max(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: LargestAllocation = LargestAllocation;

#[test]
fn string_with_huge_length_prefix_allocates_only_what_it_reads() {
    let mut stream = Cursor::new([0xff, 0xff, 0xff, 0xff, 0x0f, b'h', b'i']);
    LARGEST.store(0, Ordering::Relaxed);
    let result = String::deserialize(&mut stream);
    let largest = LARGEST.load(Ordering::Relaxed);
    assert!(result.is_err(), "expected an error, got {result:?}");
    assert!(
        largest < 1024,
        "allocated {largest} bytes for a 2-byte string body"
    );
}
