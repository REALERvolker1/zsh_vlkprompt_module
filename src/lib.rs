#![no_std]
#![allow(static_mut_refs)]
extern crate alloc;

pub mod printing;
pub mod wrapper;

use crate::printing::stdout_println;
use ::alloc::boxed::Box;
use ::core::{
    ffi::{CStr, c_char, c_int},
    ptr::null_mut,
};
use ::heapless::String;
use ::zsh_sys::{
    builtin, features, featuresarray, handlefeatures, module, options, paramdef, setfeatureenables,
    wrappers,
};

mod zallocator {
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
}

#[panic_handler]
fn p(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut infostr = String::<256>::new();
    // TODO: Remove core::fmt when the time is right
    _ = core::fmt::write(&mut infostr, format_args!("{}", info)).map(|_| stdout_println(&infostr));
    unsafe { libc::exit(libc::EXIT_FAILURE) };
}

struct Globals {
    pub patab: &'static mut [paramdef],
    pub bintab: &'static mut [builtin],
}
unsafe impl Send for Globals {}
unsafe impl Sync for Globals {}

static mut GLOBALS: Globals = Globals {
    patab: &mut [],
    bintab: &mut [builtin::BUILTIN(
        c"example".as_ptr().cast_mut(),
        0,
        Some(bin_example),
        0,
        4,
        0,
        c"flags".as_ptr().cast_mut(),
        null_mut(),
    )],
};

unsafe extern "C" fn bin_example(
    nam: *mut c_char,
    mut args: *mut *mut c_char,
    opts: *mut options,
    func: c_int,
) -> c_int {
    let oargs = args;
    stdout_println("Options:");
    (32..128).for_each(|c| {
        // TODO: Add wrapper for this:
        // #define OPT_ISSET(ops, c) ((ops)->ind[c] != 0)
        if unsafe { (*opts).ind[c] } != 0 {
            unsafe { libc::putchar_unlocked(c as c_int) };
        }
    });
    stdout_println("\nArguments:");

    unsafe {
        while !(*args).is_null() {
            args = args.add(1);
            libc::putchar_unlocked(b' ' as _);
            libc::write(
                libc::STDOUT_FILENO,
                (*args).cast(),
                zsh_sys::ztrlen(*args) as _,
            );
        }
    }
    stdout_println("\nName:");
    unsafe { libc::write(libc::STDOUT_FILENO, nam.cast(), libc::strlen(nam)) };

    // while !unsafe { (*args) }.is_null() {
    //     args = args.wrapping_add(1);

    // }
    0
}

struct Static<T>(pub T);
impl<T> Static<T> {
    pub const fn ptr(&mut self) -> *mut T {
        &raw mut self.0
    }
}
unsafe impl<T> Send for Static<T> {}
unsafe impl<T> Sync for Static<T> {}

static mut FEATURES: Static<features> = Static(features {
    bn_list: unsafe { GLOBALS.bintab.as_mut_ptr() },
    bn_size: unsafe { GLOBALS.bintab.len() as _ },
    cd_list: null_mut(),
    cd_size: 0,
    mf_list: null_mut(),
    mf_size: 0,
    pd_list: null_mut(),
    pd_size: 0,
    n_abstract: 0,
});

/// Initial memory allocation and basic setup. Called before dependencies are checked.
#[unsafe(no_mangle)]
pub extern "C" fn setup_(m: *mut module) -> c_int {
    stdout_println("The example module has now been set up.");
    // let mymod = unsafe { module::M };
    0
}
/// Final initialization. Features are registered here (e.g., [`addbuiltins`](zsh_sys::addbuiltins)).
#[unsafe(no_mangle)]
pub extern "C" fn boot_(m: *mut module) -> c_int {
    // let me = unsafe { Box::from_raw(module) };
    0
}
/// Prepares for unloading. Unregisters features.
#[unsafe(no_mangle)]
pub extern "C" fn cleanup_(m: *mut module) -> c_int {
    return unsafe { setfeatureenables(m, FEATURES.ptr(), null_mut()) };
    // let me = unsafe { Box::from_raw(module) };
    0
}
/// Final cleanup before memory deallocation.
#[unsafe(no_mangle)]
pub extern "C" fn finish_(module: *mut module) -> c_int {
    stdout_println("Thank you for using the example module.  Have a nice day.");
    // let me = unsafe { Box::from_raw(module) };
    0
}
/// Returns a list of strings identifying features (builtins, params, etc.) provided.
#[unsafe(no_mangle)]
pub extern "C" fn features_(m: *mut module, features: *mut *mut *mut c_char) -> c_int {
    unsafe { *features = featuresarray(m, FEATURES.ptr()) };
    // let me = unsafe { Box::from_raw(module) };
    0
}
/// Allows the shell to enable or disable specific features within the module.
#[unsafe(no_mangle)]
pub extern "C" fn enables_(m: *mut module, enables: *mut *mut c_int) -> c_int {
    unsafe { handlefeatures(m, FEATURES.ptr(), enables) }
}
