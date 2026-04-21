use {
    ::alloc::boxed::Box,
    ::core::{
        mem::{ManuallyDrop, MaybeUninit},
        ptr::{NonNull, null_mut},
    },
    ::zsh_sys::{hashnode, module, module__bindgen_ty_1, zalloc, zfree},
};

pub struct Module {
    /// SAFETY: We can do `'static mut` because this lives for the duration of the library
    inner: &'static mut module,
}
impl Module {
    fn util_make_module() -> module {
        module {
            autoloads: null_mut(),
            deps: null_mut(),
            wrapper: 0,
            node: hashnode {
                flags: 0,
                next: null_mut(),
                nam: null_mut(),
            },
            u: module__bindgen_ty_1 {
                alias: ManuallyDrop::new(null_mut()),
            },
        }
    }

    /// # Safety
    /// Caller asserts this is called in `setup_`
    pub unsafe fn init(ptr: *mut module) -> Self {
        assert!(ptr.is_null());

        // SAFETY: zsh throws a fatal error if we can't allocate, zsh internally uses a critical section to allocate here
        let ptr_alloc: *mut MaybeUninit<module> = unsafe { zalloc(size_of::<module>()) }.cast();
        let inner = unsafe { ptr_alloc.as_mut_unchecked() }.write(Self::util_make_module());

        Self { inner }
    }
    pub unsafe fn delete(self) {
        fn free_member<T>(ptr: *mut T) {
            if ptr.is_null() {
                return;
            }
            let vp = ptr.cast();
            let sz = size_of::<T>() as _;
            unsafe { zfree(vp, sz) }
        }

        free_member(self.inner.autoloads);
        free_member(self.inner.deps);
        // TODO: We leak hashtables
        // free_member(self.inner.node.nam);
        // free_member(self.inner.node.next);
        // drop(self.inner.u);
    }
}
impl Drop for Module {
    fn drop(&mut self) {}
}
