#[allow(unused_imports)]
use super::*;
use crate::*;

pub struct MetaString {
    pub ptr: *mut c_char,
}
impl MetaString {
    #[inline(always)]
    pub unsafe fn nicezputs(&self, outs: *mut libc::FILE) {
        unsafe { mb_niceformat(self.ptr, outs, null_mut(), 0) };
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARINIT() {
        unsafe { mb_charinit() };
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARLENCONV(&self, wcp: *mut convchar_t) -> c_int {
        unsafe { mb_metacharlenconv(self.ptr, wcp) }
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARLEN(&self) -> c_int {
        unsafe { self.MB_METACHARLENCONV(null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRLEN(&self) -> c_int {
        unsafe { mb_metastrlenend(self.ptr, 0, null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRWIDTH(&self) -> c_int {
        unsafe { mb_metastrlenend(self.ptr, 1, null_mut()) }
    }
    #[inline(always)]
    pub fn MB_METASTRLEN2(&self, widthp: c_int) -> c_int {
        unsafe { mb_metastrlenend(self.ptr, widthp, null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRLEN2END(&self, widthp: c_int, eptr: &Self) -> i32 {
        unsafe { mb_metastrlenend(self.ptr, widthp, eptr.ptr) }
    }
}
pub use crate::mb_charinit as MB_CHARINIT;
