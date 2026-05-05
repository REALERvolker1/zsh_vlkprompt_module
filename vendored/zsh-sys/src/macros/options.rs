#[allow(unused_imports)]
use super::*;
use crate::*;

pub fn EMULATION(X: c_int) -> c_int {
    let emu = unsafe { emulation };
    emu & X
}
const EMULATE_FIELDS: c_int = EMULATE_CSH | EMULATE_KSH | EMULATE_SH | EMULATE_ZSH;
/// Return only base shell emulation field.
#[inline(always)]
pub fn SHELL_EMULATION() -> c_int {
    EMULATION(EMULATE_FIELDS)
}

impl OPT {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
    #[inline(always)]
    pub const fn as_optindex(self) -> OptIndex {
        self as _
    }
    #[inline(always)]
    const fn as_index(self) -> usize {
        self as _
    }
    pub fn isset(self) -> c_char {
        let idx = self.as_index();
        // SAFETY: We know it is in bounds
        let val = unsafe { opts[idx] };
        val
    }
    #[inline(always)]
    pub fn unset(self) -> c_char {
        !self.isset()
    }

    /// Set or unset an option, as a result of user request.
    /// # Safety
    /// Caller asserts no contention.
    pub unsafe fn setoption(self, value: c_int, force: bool) {
        unsafe { dosetopt(self.as_int(), value, force as _, opts.as_mut_ptr()) };
    }

    pub unsafe fn optlookupc(c: c_char) -> Self {
        // SAFETY: The lookup table is defined in const in zsh itself
        unsafe { transmute(optlookupc(c)) }
    }
    pub unsafe fn optlookup(name: *const c_char) -> Self {
        // SAFETY: The lookup table is defined in const in zsh itself
        unsafe { transmute(optlookup(name)) }
    }
}

pub unsafe fn tccan(X: c_int) -> c_int {
    let idx = X as usize;
    unsafe { tclen[idx] }
}
