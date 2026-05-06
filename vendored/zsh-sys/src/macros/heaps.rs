#[allow(unused_imports)]
use super::*;
use {
    crate::*,
    ::bytemuck::Pod,
    ::core::{
        marker::PhantomData,
        mem::MaybeUninit,
        num::NonZeroUsize,
        ops::{Index, IndexMut},
        ptr::NonNull,
    },
};

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

/// A safe(ish) buffer for using zsh's scratchpad buffers
///
/// # Safety
/// This can only be safely initialized/interacted with from the zsh thread.
///
/// This type does **not** implement `Drop` because zsh's heap system does not allow it safely.
pub struct ZHeapVec<T> {
    ptr: NonNull<MaybeUninit<T>>,
    /// zsh's heap system cannot handle zero-capacity buffers
    cap: NonZeroUsize,
    populated: usize,
    /// Variance, make this single-threaded
    _marker: PhantomData<*const ()>,
}
impl<T> ZHeapVec<T> {
    /// This can be pretty big, zsh's scratch heaps are massive pages
    const DEFAULT_CAP: NonZeroUsize = NonZeroUsize::new(256).unwrap();

    pub const fn len(&self) -> usize {
        self.populated
    }
    pub const fn capacity(&self) -> NonZeroUsize {
        self.cap
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// # Safety
    /// Initializing this type outside of the main zsh thread is Undefined Behavior.
    pub unsafe fn new() -> Option<Self> {
        unsafe { Self::with_capacity(Self::DEFAULT_CAP) }
    }
    /// # Safety
    /// Initializing this type outside of the main zsh thread is Undefined Behavior.
    pub unsafe fn with_capacity(size: NonZeroUsize) -> Option<Self> {
        let bytecap = size.get() * size_of::<T>();
        let ptr = NonNull::new(unsafe { zhalloc(bytecap) }.cast())?;

        Some(Self {
            ptr,
            cap: size,
            populated: 0,
            _marker: PhantomData,
        })
    }

    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: This type is !Send, and the caller initialized it from zsh-main (with unsafe code)
        // We also assert the slice is populated from `0..len`
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len()) }
    }
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: This type is !Send, and the caller initialized it from zsh-main (with unsafe code)
        // We also assert the slice is populated from `0..len`
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr().cast(), self.len()) }
    }

    /// Drop order is front-to-back
    pub fn clear(&mut self) {}

    const unsafe fn _index_ref(&self, i: usize) -> &MaybeUninit<T> {
        unsafe { self.ptr.add(i).as_ref() }
    }
    const unsafe fn _index_mut(&mut self, i: usize) -> &mut MaybeUninit<T> {
        unsafe { self.ptr.add(i).as_mut() }
    }
    const unsafe fn _index_init_ref(&self, i: usize) -> &T {
        unsafe { self._index_ref(i).assume_init_ref() }
    }
    const unsafe fn _index_init_mut(&mut self, i: usize) -> &mut T {
        unsafe { self._index_mut(i).assume_init_mut() }
    }

    unsafe fn _hrealloc(&mut self, new_cap: NonZeroUsize) -> Result<&mut Self, &mut Self> {
        use core::cmp::Ordering;

        match self.capacity().cmp(&new_cap) {
            Ordering::Equal => return Ok(self),
            Ordering::Less => return Err(self),
            Ordering::Greater => {}
        }

        let old_cap = self.capacity().get() * size_of::<T>();
        let new_cap = new_cap.get() * size_of::<T>();
        let cp: *mut c_char = self.ptr.as_ptr().cast();

        // SAFETY: Caller asserts this was constructed on the zsh thread (unsafe constructors)
        // We assert this is *still* on the zsh thread due to the !Send bound
        let new_p = NonNull::new(unsafe { hrealloc(cp, old_cap, new_cap) }.cast());

        match new_p {
            // Don't leak memory, return the old one if invalid
            None => Err(self),
            Some(p) => {
                self.ptr = p;
                Ok(self)
            }
        }
    }

    const fn _calc_new_cap(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.capacity().get() << 1).expect("New capacity calc overflowed")
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len() >= self.capacity().get() {
            let ncap = self._calc_new_cap();
            if unsafe { self._hrealloc(ncap) }.is_err() {
                return Err(value);
            }
        }
        unsafe { self._index_mut(self.len()) }.write(value);
        self.populated += 1;

        Ok(())
    }
    pub fn pop(&mut self) -> Option<T> {
        let new_len = self.len().checked_sub(1)?;
        self.populated = new_len;

        let val = unsafe { self._index_mut(new_len).assume_init_read() };
        Some(val)
    }
}
impl<T> Index<usize> for ZHeapVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T> IndexMut<usize> for ZHeapVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}
