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

const STDOUT_FD: BorrowedFd<'static> = unsafe { BorrowedFd::borrow_raw(STDOUT_FILENO) };
const STDERR_FD: BorrowedFd<'static> = unsafe { BorrowedFd::borrow_raw(STDERR_FILENO) };

#[inline]
pub fn stdout_write_blocking(buf: &[u8]) -> Result<usize, io::Errno> {
    write_all(STDOUT_FD, buf)
}
#[inline]
pub fn stderr_write_blocking(buf: &[u8]) -> Result<usize, io::Errno> {
    write_all(STDERR_FD, buf)
}

#[inline]
pub fn stdout_println(s: &str) {
    _ = stdout_write_blocking(s.as_bytes()).and_then(|_| rustix::io::write(STDOUT_FD, b"\n"));
}
