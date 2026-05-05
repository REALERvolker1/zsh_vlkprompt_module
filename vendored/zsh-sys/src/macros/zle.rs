#[allow(unused_imports)]
use super::*;

pub const WIDGET_INT: c_int = 1 << 0;
pub const WIDGET_NCOMP: c_int = 1 << 1;
pub const ZLE_MENUCMP: c_int = 1 << 2;
pub const ZLE_YANKAFTER: c_int = 1 << 3;
pub const ZLE_YANKBEFORE: c_int = 1 << 4;
pub const ZLE_YANK: c_int = ZLE_YANKAFTER | ZLE_YANKBEFORE;
pub const ZLE_LINEMOVE: c_int = 1 << 5;
pub const ZLE_VIOPER: c_int = 1 << 6;
pub const ZLE_LASTCOL: c_int = 1 << 7;
pub const ZLE_KILL: c_int = 1 << 8;
pub const ZLE_KEEPSUFFIX: c_int = 1 << 9;
pub const ZLE_NOTCOMMAND: c_int = 1 << 10;
pub const ZLE_ISCOMP: c_int = 1 << 11;
pub const WIDGET_INUSE: c_int = 1 << 12;
pub const WIDGET_FREE: c_int = 1 << 13;
pub const ZLE_NOLAST: c_int = 1 << 14;

pub const TH_IMMORTAL: c_int = 1 << 1;

pub const MOD_MULT: c_int = 1 << 0;
pub const MOD_TMULT: c_int = 1 << 1;
pub const MOD_VIBUF: c_int = 1 << 2;
pub const MOD_VIAPP: c_int = 1 << 3;
pub const MOD_NEG: c_int = 1 << 4;
pub const MOD_NULL: c_int = 1 << 5;
pub const MOD_CHAR: c_int = 1 << 6;
pub const MOD_LINE: c_int = 1 << 7;
pub const MOD_PRI: c_int = 1 << 8;
pub const MOD_CLIP: c_int = 1 << 9;
pub const MOD_OSSEL: c_int = MOD_PRI | MOD_CLIP;

pub const CUT_FRONT: c_int = 1 << 0;
pub const CUT_REPLACE: c_int = 1 << 1;
pub const CUT_RAW: c_int = 1 << 2;
pub const CUT_YANK: c_int = 1 << 3;

pub const CH_NEXT: c_int = 1 << 0;
pub const CH_PREV: c_int = 1 << 1;

pub const CUTBUFFER_LINE: c_char = 1;
pub const KRINGCTDEF: c_int = 8;

pub const COMP_COMPLETE: c_int = 0;
pub const COMP_LIST_COMPLETE: c_int = 1;
pub const COMP_SPELL: c_int = 2;
pub const COMP_EXPAND: c_int = 3;
pub const COMP_EXPAND_COMPLETE: c_int = 4;
pub const COMP_LIST_EXPAND: c_int = 5;

#[inline(always)]
pub const fn COMP_ISEXPAND(x: c_int) -> bool {
    x >= COMP_EXPAND
}

pub const ZSL_COPY: c_int = 1;
pub const ZSL_TOEND: c_int = 2;

pub const SUFTYP_POSSTR: c_int = 0;
pub const SUFTYP_NEGSTR: c_int = 1;
pub const SUFTYP_POSRNG: c_int = 2;
pub const SUFTYP_NEGRNG: c_int = 3;

pub const SUFFLAGS_SPACE: c_int = 0x0001;

pub const ZRH_PREDISPLAY: c_int = 1;
pub const N_SPECIAL_HIGHLIGHTS: c_int = 4;

pub const CURC_EDIT: c_int = 0;
pub const CURC_COMMAND: c_int = 1;
pub const CURC_INSERT: c_int = 2;
pub const CURC_OVERWRITE: c_int = 3;
pub const CURC_PENDING: c_int = 4;
pub const CURC_REGION_START: c_int = 5;
pub const CURC_REGION_END: c_int = 6;
pub const CURC_VISUAL: c_int = 7;
pub const CURC_DEFAULT: c_int = 8;

pub const CURF_DEFAULT: c_int = 0;
pub const CURF_UNDERLINE: c_int = 1;
pub const CURF_BAR: c_int = 2;
pub const CURF_BLOCK: c_int = 3;
pub const CURF_SHAPE_MASK: c_int = 3;
pub const CURF_BLINK: c_int = 1 << 2;
pub const CURF_STEADY: c_int = 1 << 3;
pub const CURF_HIDDEN: c_int = 1 << 4;
pub const CURF_COLOR: c_int = 1 << 5;
pub const CURF_COLOR_MASK: c_int = ((0x00ff_ffffu32 << 8) as c_int) | CURF_COLOR;
pub const CURF_RED_SHIFT: c_int = 24;
pub const CURF_GREEN_SHIFT: c_int = 16;
pub const CURF_BLUE_SHIFT: c_int = 8;

#[inline]
pub unsafe fn zle_hook(name: *const c_char) -> *mut hookdef {
    unsafe { gethookdef(name) }
}

#[inline]
pub unsafe fn LISTMATCHESHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"list_matches".as_ptr()) }
}

#[inline]
pub unsafe fn COMPLETEHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"complete".as_ptr()) }
}

#[inline]
pub unsafe fn BEFORECOMPLETEHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"before_complete".as_ptr()) }
}

#[inline]
pub unsafe fn AFTERCOMPLETEHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"after_complete".as_ptr()) }
}

#[inline]
pub unsafe fn ACCEPTCOMPHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"accept_completion".as_ptr()) }
}

#[inline]
pub unsafe fn INVALIDATELISTHOOK() -> *mut hookdef {
    unsafe { zle_hook(c"invalidate_list".as_ptr()) }
}

#[inline]
pub unsafe fn listmatches() -> c_int {
    let hook = unsafe { LISTMATCHESHOOK() };
    if hook.is_null() {
        return 1;
    }
    unsafe { runhookdef(hook, null_mut()) }
}

#[inline]
pub unsafe fn invalidatelist() -> c_int {
    let hook = unsafe { INVALIDATELISTHOOK() };
    if hook.is_null() {
        return 1;
    }
    unsafe { runhookdef(hook, null_mut()) }
}
