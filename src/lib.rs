#![no_std]
#![allow(static_mut_refs)]

extern crate alloc;

mod paramtest;
pub mod printing;
pub mod wrapper;
mod zalloc;

use crate::{printing::stdout_println, wrapper::Static};
use ::core::{
    ffi::{c_char, c_int},
    ptr::null_mut,
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

unsafe extern "C" fn bin_example2(
    _nam: *mut c_char,
    _args: *mut *mut c_char,
    _opts: *mut options,
    _func: c_int,
) -> c_int {
    unsafe { paramtest::run_parameter_tests() as c_int }
}

mod excount_experiments {
    use zsh_sys::zlong;

    pub static mut EXCOUNT: zlong = 0;
}

static mut PARAMTAB: Static<[paramdef; 1]> = Static([paramdef::INTPARAMDEF(
    c"EXCOUNT".as_ptr().cast_mut(),
    &raw mut excount_experiments::EXCOUNT,
)]);

static mut BINTAB: Static<[builtin; 1]> = Static([builtin::BUILTIN(
    c"example".as_ptr().cast_mut(),
    0,
    Some(bin_example2),
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
    unsafe { setfeatureenables(m, FEATURES.ptr(), null_mut()) }
}

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
