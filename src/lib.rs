#![no_std]

pub mod printing;

use crate::printing::{stderr_write_blocking, stdout_println, stdout_write_blocking};
use ::core::ffi::CStr;
use ::heapless::{String, Vec};
use bytemuck::{Pod, TransparentWrapper, Zeroable};
use core::ffi::{c_char, c_int, c_long, c_void};
use core::num::{NonZero, NonZeroUsize};
use libc::{size_t, ssize_t};
use owo_colors::Color;
use zsh_sys::{zlong, zulong};

#[panic_handler]
fn p(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut infostr = String::<256>::new();
    // TODO: Remove core::fmt when the time is right
    _ = core::fmt::write(&mut infostr, format_args!("{}", info)).map(|_| stdout_println(&infostr));
    unsafe { libc::exit(libc::EXIT_FAILURE) };
}

/// Initial memory allocation and basic setup. Called before dependencies are checked.
#[unsafe(no_mangle)]
pub extern "C" fn setup_(module: zsh_sys::Module) -> c_int {
    0
}
/// Final initialization. Features are registered here (e.g., [`addbuiltins`](zsh_sys::addbuiltins)).
#[unsafe(no_mangle)]
pub extern "C" fn boot_(module: zsh_sys::Module) -> c_int {
    0
}
/// Prepares for unloading. Unregisters features.
#[unsafe(no_mangle)]
pub extern "C" fn cleanup_(module: zsh_sys::Module) -> c_int {
    0
}
/// Final cleanup before memory deallocation.
#[unsafe(no_mangle)]
pub extern "C" fn finish_(module: zsh_sys::Module) -> c_int {
    0
}
/// Returns a list of strings identifying features (builtins, params, etc.) provided.
#[unsafe(no_mangle)]
pub extern "C" fn features_(module: zsh_sys::Module, features: *mut *mut *mut c_char) -> c_int {
    0
}
/// Allows the shell to enable or disable specific features within the module.
#[unsafe(no_mangle)]
pub extern "C" fn enables_(module: zsh_sys::Module, enables: *mut *mut c_int) -> c_int {
    0
}
