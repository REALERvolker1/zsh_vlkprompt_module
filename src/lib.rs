#![no_std]

pub mod printing;

use crate::printing::{stderr_write_blocking, stdout_println, stdout_write_blocking};
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

#[unsafe(no_mangle)]
pub extern "C" fn setup_(module: zsh_sys::Module) -> c_int {
    // stdout_println("setup_");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn boot_(module: zsh_sys::Module) -> c_int {
    // stdout_println("boot_");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn cleanup_(module: zsh_sys::Module) -> c_int {
    // stdout_println("cleanup_");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn finish_(module: zsh_sys::Module) -> c_int {
    // stdout_println("finish_");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn features_(module: zsh_sys::Module, features: *mut *mut *mut c_char) -> c_int {
    // stdout_println("features_");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn enables_(module: zsh_sys::Module, enables: *mut *mut c_int) -> c_int {
    // stdout_println("enables_");
    0
}
