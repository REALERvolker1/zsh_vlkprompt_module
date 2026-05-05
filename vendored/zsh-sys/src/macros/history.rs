#[allow(unused_imports)]
use super::*;
use crate::*;

pub unsafe fn firsthist() -> i64 {
    let r = unsafe { hist_ring.as_mut() };
    let Some(ring) = r else {
        return unsafe { curhist };
    };
    unsafe { ring.down.read() }.histnum
}

/// ```c
/// #define sigmsg(sig) ((sig) <= SIGCOUNT ? sig_msg[sig] : "unknown signal")

pub fn STOPHIST() {
    unsafe { stophist += 4 }
}
#[inline]
pub fn ALLOWHIST() {
    unsafe { stophist -= 4 }
}
