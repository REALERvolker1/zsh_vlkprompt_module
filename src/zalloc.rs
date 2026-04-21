use {
    ::core::alloc::{GlobalAlloc, Layout},
    ::zsh_sys::{zalloc, zfree, zrealloc, zshcalloc},
};

#[global_allocator]
static ZALLOCATOR: Zallocator = Zallocator;

struct Zallocator;
unsafe impl GlobalAlloc for Zallocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { zalloc(layout.size()) }.cast()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { zfree(ptr.cast(), layout.size() as _) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, _: Layout, new_size: usize) -> *mut u8 {
        unsafe { zrealloc(ptr.cast(), new_size as _) }.cast()
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { zshcalloc(layout.size()) }.cast()
    }
}
