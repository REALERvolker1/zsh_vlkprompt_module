#![allow(nonstandard_style, static_mut_refs)]

//! C macro compatibility helpers, organized by the zsh subsystem they mirror.
//!
//! `build.rs` overwrites only `src/bindings.rs`; these hand-written modules are
//! intentionally kept outside the generated file.  Most functions retain the
//! zsh macro names (`BUILTIN`, `PM_TYPE`, `STOPHIST`, etc.) so code can be read
//! side-by-side with upstream C examples.

#[allow(unused_imports)]
pub(crate) use ::core::{
    ffi::{CStr, c_char, c_int, c_void},
    mem::transmute,
    ops::Add,
    ptr::{null, null_mut},
};

#[allow(unused_imports)]
pub(crate) use ::libc::{SIG_DFL, SIG_IGN, SIGRTMAX, SIGRTMIN, SIGWINCH, signal, sigset_t, strcmp};

#[allow(unused_imports)]
pub(crate) use crate::{
    ASG, BINF_PREFIX, CondHandler, EF_HEAP, EF_MAP, EF_REAL, EF_RUN, EMULATE_CSH, EMULATE_KSH,
    EMULATE_SH, EMULATE_ZSH, GetNodeFunc, HandlerFunc, Heapid, Hookfn, MAX_QUEUE_SIZE, MFF_STR,
    MN_FLOAT, MN_INTEGER, MN_UNSET, NumMathFunc, OPT, OptIndex, PM_ARRAY, PM_DECLARED, PM_EFLOAT,
    PM_READONLY, PM_REMOVABLE, PM_RESTRICTED, PM_SCALAR, PM_SPECIAL, PM_TIED, PM_UNSET, QT, REDIR,
    SIGCOUNT, ScanTabFunc, StrMathFunc, VALFLAG_EMPTY, VALFLAG_INV, VALFLAG_REFSLICE,
    VALFLAG_SUBST, VSIGCOUNT, WrapFunc, addparamdef, asgment, assignaparam, assigniparam,
    assignsparam, builtin, features, funcwrap, getaparam, getarrvalue, gethashnode, gethashnode2,
    gethookdef, gethparam, gsu_float, gsu_hash, gsu_integer, gsu_scalar, hashnode, hashtable, heap,
    hist_ring, hookdef, intrap, lextok, linklist, linknode, locallevel, matheval, mathfunc,
    mb_charinit, mb_metacharlenconv, mb_metastrlenend, mb_niceformat, mnumber,
    mnumber__bindgen_ty_1, new_heaps, old_heaps, optlookup, optlookupc, opts, param, paramdef,
    paramtab, queue_front, queue_in, queue_rear, queueing_enabled, realparamtab, resetparam,
    runhookdef, setaparam, setarrvalue, sethparam, setiparam, setiparam_no_convert, setnparam,
    setnumvalue, setsparam, setstrvalue, sig_msg, sigchld_mask, signal_block, signal_mask,
    signal_mask_queue, signal_queue, signal_setmask, signal_unblock, stophist, switch_heaps, tclen,
    trapisfunc, traplocallevel, unsetparam, unsetparam_pm, value, zero_mnumber, zhandler, zlong,
    zshhooks,
};

pub mod arithmetic;
pub mod assignment;
pub mod builtins;
pub mod conditions;
pub mod feature_sets;
pub mod heaps;
pub mod history;
pub mod hooks;
pub mod lists;
pub mod options;
pub mod params;
pub mod signals;
pub mod strings;
pub mod syntax;
pub mod zle;

pub use heaps::*;
pub use history::*;
pub use hooks::*;
pub use options::*;
pub use params::*;
pub use signals::*;
pub use strings::*;
pub use zle::*;
