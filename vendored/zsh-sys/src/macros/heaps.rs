use super::*;

pub const fn arena(X: Heapid) -> *mut char {
    core::ptr::without_provenance_mut((X as usize) + size_of::<heap>())
}

// Those macros confuse me. This is less confusing.

/// Run a closure with a new temporary heap lasting as long as the closure.
/// # Safety
/// This function is *neither signal-safe, nor thread-safe*.
#[inline]
pub unsafe fn with_new_heap<R>(f: impl FnOnce() -> R) -> R {
    let old_heap = unsafe { new_heaps() };

    let res = f();

    unsafe { old_heaps(old_heap) };
    res
}
/// Run a closure with an arbitrary heap.
/// # Safety
/// This function is *neither signal-safe, nor thread-safe*.
#[inline]
pub unsafe fn with_heap<R>(h: *mut heap, f: impl FnOnce() -> R) -> R {
    let o = unsafe { switch_heaps(h) };

    let res = f();

    unsafe { switch_heaps(o) };
    res
}
