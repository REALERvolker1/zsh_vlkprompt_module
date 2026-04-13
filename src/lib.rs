#![no_std]

pub mod printing;

use crate::printing::{stderr_write_blocking, stdout_write_blocking};
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
    if core::fmt::write(&mut infostr, format_args!("{}", info)).is_ok() {
        let slen = infostr.len();
    }
    unsafe { libc::exit(libc::EXIT_FAILURE) };
}
