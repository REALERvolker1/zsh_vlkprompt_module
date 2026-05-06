use {crate::convchar_t, ::core::ptr::NonNull};

use super::*;
use ::bytemuck::TransparentWrapper;

#[derive(Debug, TransparentWrapper, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MetaString {
    ptr: NonNull<c_char>,
}
impl MetaString {
    pub const fn from_ptr(ptr: *mut c_char) -> Option<Self> {
        match NonNull::new(ptr) {
            None => None,
            Some(ptr) => Some(Self { ptr }),
        }
    }
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const c_char {
        self.ptr.as_ptr().cast_const()
    }
    #[inline(always)]
    pub const fn as_mut_ptr(&mut self) -> *mut c_char {
        self.ptr.as_ptr()
    }
    #[inline(always)]
    pub const fn as_nonnull_ptr(&mut self) -> NonNull<c_char> {
        self.ptr
    }
    #[inline(always)]
    pub unsafe fn nicezputs(&self, outs: *mut libc::FILE) {
        unsafe { mb_niceformat(self.as_ptr(), outs, null_mut(), 0) };
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARINIT() {
        unsafe { mb_charinit() };
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARLENCONV(&self, wcp: *mut convchar_t) -> c_int {
        unsafe { mb_metacharlenconv(self.as_ptr(), wcp) }
    }
    #[inline(always)]
    pub unsafe fn MB_METACHARLEN(&self) -> c_int {
        unsafe { self.MB_METACHARLENCONV(null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRLEN(&mut self) -> c_int {
        unsafe { mb_metastrlenend(self.as_mut_ptr(), 0, null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRWIDTH(&mut self) -> c_int {
        unsafe { mb_metastrlenend(self.as_mut_ptr(), 1, null_mut()) }
    }
    #[inline(always)]
    pub fn MB_METASTRLEN2(&mut self, widthp: c_int) -> c_int {
        unsafe { mb_metastrlenend(self.as_mut_ptr(), widthp, null_mut()) }
    }
    #[inline(always)]
    pub unsafe fn MB_METASTRLEN2END(&mut self, widthp: c_int, eptr: &mut Self) -> i32 {
        unsafe { mb_metastrlenend(self.as_mut_ptr(), widthp, eptr.as_mut_ptr()) }
    }
}
pub use crate::mb_charinit as MB_CHARINIT;
