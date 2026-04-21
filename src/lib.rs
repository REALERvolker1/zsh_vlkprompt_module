#![no_std]
#![allow(static_mut_refs)]
extern crate alloc;

pub mod printing;
pub mod wrapper;
mod zalloc;

use crate::{printing::stdout_println, wrapper::Static};
use ::core::{
    ffi::{c_char, c_int},
    ptr::null_mut,
    sync::atomic::AtomicI64,
};
use ::heapless::String;
use ::zsh_sys::{
    builtin, features, featuresarray, handlefeatures, module, options, paramdef, setfeatureenables,
};

#[panic_handler]
fn p(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut infostr = String::<256>::new();
    // TODO: Remove core::fmt when the time is right
    _ = core::fmt::write(&mut infostr, format_args!("{}", info)).map(|_| stdout_println(&infostr));
    unsafe { libc::exit(libc::EXIT_FAILURE) };
}

unsafe extern "C" fn bin_example(
    nam: *mut c_char,
    mut args: *mut *mut c_char,
    opts: *mut options,
    _func: c_int,
) -> c_int {
    let old = EXCOUNT.fetch_add(1, ::core::sync::atomic::Ordering::AcqRel);
    let _oargs = args;
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
        while !args.is_null() && !(*args).is_null() {
            libc::putchar_unlocked(b' ' as _);
            libc::write(
                libc::STDOUT_FILENO,
                (*args).cast(),
                zsh_sys::ztrlen(*args) as _,
            );
            args = args.add(1);
        }
    }
    stdout_println("\nName:");
    unsafe { libc::write(libc::STDOUT_FILENO, nam.cast(), libc::strlen(nam)) };

    stdout_println("\nCount:");
    unsafe { libc::printf(c"%u\n".as_ptr(), old) };
    // while !unsafe { (*args) }.is_null() {
    //     args = args.wrapping_add(1);

    // }
    0
}

static EXCOUNT: AtomicI64 = AtomicI64::new(0);

static mut PARAMTAB: Static<[paramdef; 1]> = Static([paramdef::INTPARAMDEF(
    c"EXCOUNT".as_ptr().cast_mut(),
    EXCOUNT.as_ptr(),
)]);

static mut BINTAB: Static<[builtin; 1]> = Static([builtin::BUILTIN(
    c"example".as_ptr().cast_mut(),
    0,
    Some(bin_example),
    0,
    5,
    0,
    c"flags".as_ptr().cast_mut(),
    null_mut(),
)]);
static mut FEATURES: Static<features> = Static(
    features::CONST_DEFAULT
        .with_builtins(unsafe { BINTAB.0.as_mut_slice() })
        .with_params(unsafe { PARAMTAB.0.as_mut_slice() }),
);

/// Initial memory allocation and basic setup. Called before dependencies are checked.
#[unsafe(no_mangle)]
pub extern "C" fn setup_(_m: *mut module) -> c_int {
    stdout_println("The example module has now been set up.");
    // let mymod = unsafe { module::M };
    0
}
/// Final initialization. Features are registered here (e.g., [`addbuiltins`](zsh_sys::addbuiltins)).
#[unsafe(no_mangle)]
pub extern "C" fn boot_(_m: *mut module) -> c_int {
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
pub extern "C" fn finish_(_module: *mut module) -> c_int {
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
