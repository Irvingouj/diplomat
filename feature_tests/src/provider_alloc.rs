//! Counts heap allocations made by this provider library, then forwards them
//! to the system allocator. The Rust fixture reads the count to see that
//! dropping an owned byte buffer frees it here, not in the caller.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);

pub fn live() -> usize {
    LIVE.load(Ordering::SeqCst)
}

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: `layout` is the layout the caller asked to allocate.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && layout.size() > 0 {
            LIVE.fetch_add(1, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() > 0 {
            LIVE.fetch_sub(1, Ordering::SeqCst);
        }
        // SAFETY: `ptr` came from this allocator with `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: `layout` is the layout the caller asked to allocate.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && layout.size() > 0 {
            LIVE.fetch_add(1, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: `ptr` came from this allocator with `layout`. A successful
        // realloc is still one live allocation.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;
