use ::core::ffi::c_int;

use ::libc::{STDERR_FILENO, STDOUT_FILENO};
use ::rustix::{fd::BorrowedFd, io};

fn write_all(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let mut i = 0;
    loop {
        let written = rustix::io::write(fd, &buf[i..])?;

        i += written;
        if i >= buf.len() {
            break Ok(i);
        }
    }
}

#[inline]
unsafe fn write_all_to_fd(fd: c_int, buf: &[u8]) -> Result<usize, io::Errno> {
    write_all(unsafe { BorrowedFd::borrow_raw(fd) }, buf)
}

#[inline]
pub fn stdout_write_blocking(buf: &[u8]) -> Result<usize, io::Errno> {
    unsafe { write_all_to_fd(STDOUT_FILENO, buf) }
}
#[inline]
pub fn stderr_write_blocking(buf: &[u8]) -> Result<usize, io::Errno> {
    unsafe { write_all_to_fd(STDERR_FILENO, buf) }
}
