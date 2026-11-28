//! How much memory a reader asks for when a palette length in the input is far larger than
//! the input. The test runs under a counting allocator, so it fails the same way whether or
//! not the machine could have given the memory.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};

use bedrock_level::Greedy;
use bedrock_level::biome::{Biomes3d, COLUMNS};

/// The system allocator, remembering the largest single request since the last reset.
struct Counting;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
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
static ALLOCATOR: Counting = Counting;

/// A biome storage whose palette length is `u32::MAX`, with no ids after it, is an error,
/// and reading it asks for no more than a storage could need, not 16 GB.
#[test]
fn biome_palette_length_beyond_the_input_is_an_error_not_an_allocation() {
    let mut bytes = vec![0u8; COLUMNS * 2];
    bytes.push(1 << 1 | 1);
    bytes.extend(std::iter::repeat_n(0u8, 128 * 4));
    bytes.extend(u32::MAX.to_le_bytes());

    LARGEST.store(0, Ordering::Relaxed);
    let result = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice()));
    let largest = LARGEST.load(Ordering::Relaxed);

    assert!(result.is_err(), "read as {result:?}");
    assert!(
        largest <= 64 * 1024,
        "the reader asked for {largest} bytes at once"
    );
}
