#![allow(nonstandard_style, static_mut_refs)]

use ::core::{
    ffi::{CStr, c_char, c_int, c_void},
    mem::transmute,
    ops::Add,
    ptr::{null, null_mut},
};

use ::libc::{SIG_DFL, SIG_IGN, SIGRTMAX, SIGRTMIN, SIGWINCH, signal, sigset_t, strcmp};

use crate::{
    ASG, BINF_PREFIX, CondHandler, EF_HEAP, EF_MAP, EF_REAL, EF_RUN, EMULATE_CSH, EMULATE_KSH,
    EMULATE_SH, EMULATE_ZSH, GetNodeFunc, HandlerFunc, Heapid, Hookfn, MAX_QUEUE_SIZE, MFF_STR,
    MN_FLOAT, MN_INTEGER, MN_UNSET, NumMathFunc, OPT, OptIndex, PM_ARRAY, PM_DECLARED, PM_EFLOAT,
    PM_EXPORTED, PM_FFLOAT, PM_HASHED, PM_HIDE, PM_HIDEVAL, PM_INTEGER, PM_LOCAL, PM_NAMEREF,
    PM_READONLY, PM_RESTRICTED, PM_SCALAR, PM_SPECIAL, PM_TIED, PM_UNSET, QT, REDIR, SIGCOUNT,
    ScanTabFunc, StrMathFunc, VALFLAG_EMPTY, VALFLAG_INV, VALFLAG_REFSLICE, VALFLAG_SUBST,
    VSIGCOUNT, WrapFunc, addparamdef, asgment, builtin, conddef, convchar_t, createparam, curhist,
    dosetopt, emulation, eprog, export_param, features, funcwrap, getarrvalue, gethookdef,
    getintvalue, getiparam, getnumvalue, getsparam, getstrvalue, gsu_array, gsu_float, gsu_hash,
    gsu_integer, gsu_scalar, hashnode, hashtable, heap, hist_ring, hookdef, intrap, lextok,
    linklist, linknode, locallevel, matheval, mathfunc, mb_charinit, mb_metacharlenconv,
    mb_metastrlenend, mb_niceformat, mnumber, mnumber__bindgen_ty_1, new_heaps, old_heaps,
    optlookup, optlookupc, opts, param, paramdef, queue_front, queue_in, queue_rear,
    queueing_enabled, resetparam, runhookdef, setnumvalue, setstrvalue, sig_msg, sigchld_mask,
    signal_block, signal_mask, signal_mask_queue, signal_queue, signal_setmask, signal_unblock,
    stophist, switch_heaps, tclen, trapisfunc, traplocallevel, unsetparam_pm, value, zero_mnumber,
    zhandler, zlong, zshhooks,
};

impl asgment {
    /// Assignment is array?
    #[inline]
    pub const fn ASG_ARRAYP(&self) -> bool {
        self.flags & ASG::ASG_ARRAY.as_int() != 0
    }
    /// Assignment has value?
    /// If the assignment is an array, then it certainly has a value --- we
    /// can only tell if there's an explicit assignment.
    #[inline]
    pub fn ASG_VALUEP(&self) -> bool {
        self.ASG_ARRAYP() || !unsafe { (*self.value.scalar).is_null() }
    }
}

pub unsafe fn firsthist() -> i64 {
    let r = unsafe { hist_ring.as_mut() };
    let Some(ring) = r else {
        return unsafe { curhist };
    };
    unsafe { ring.down.read() }.histnum
}

/// ```c
/// #define sigmsg(sig) ((sig) <= SIGCOUNT ? sig_msg[sig] : "unknown signal")
/// ```
#[inline]
pub unsafe fn sigmsg(sig: c_int) -> &'static CStr {
    if sig >= 0 && sig <= SIGCOUNT {
        // SAFETY: This is the exact same indexing rule as the C macro,
        // with an extra negative guard so accidental bad inputs do not
        // index before the exported table.
        return unsafe { CStr::from_ptr(sig_msg[sig as usize]) };
    }
    c"unknown signal"
}

impl builtin {
    #[inline]
    pub const fn BUILTIN(
        name: *mut c_char,
        flags: c_int,
        handler: HandlerFunc,
        min: c_int,
        max: c_int,
        funcid: c_int,
        optstr: *mut c_char,
        defopts: *mut c_char,
    ) -> Self {
        Self {
            node: hashnode {
                next: null_mut(),
                nam: name,
                flags: flags,
            },
            handlerfunc: handler,
            minargs: min,
            maxargs: max,
            funcid,
            optstr,
            defopts,
        }
    }
    #[inline]
    pub const fn BIN_PREFIX(name: *mut c_char, flags: c_int) -> Self {
        Self::BUILTIN(
            name,
            flags | BINF_PREFIX,
            None,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
        )
    }
}

impl funcwrap {
    pub const fn WRAPDEF(handler: WrapFunc) -> Self {
        Self {
            next: null_mut(),
            flags: 0,
            handler,
            module: null_mut(),
        }
    }
}

impl lextok {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
}
impl ASG {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
}
impl REDIR {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
    const fn is_between(self, ge: Self, le: Self) -> bool {
        let s = self.as_int();
        s >= ge.as_int() && s <= le.as_int()
    }
    pub const fn IS_WRITE_FILE(self) -> bool {
        self.is_between(REDIR::REDIR_WRITE, REDIR::REDIR_READWRITE)
    }
    pub const fn IS_APPEND_REDIR(self) -> bool {
        self.IS_WRITE_FILE() && (self.as_int() & 2 != 0)
    }
    pub const fn IS_CLOBBER_REDIR(self) -> bool {
        self.IS_WRITE_FILE() && (self.as_int() & 1 != 0)
    }
    pub const fn IS_ERROR_REDIR(self) -> bool {
        self.is_between(Self::REDIR_ERRWRITE, Self::REDIR_ERRAPPNOW)
    }
    pub const fn IS_READFD(self) -> bool {
        self.is_between(Self::REDIR_READWRITE, Self::REDIR_MERGEIN)
            || self.as_int() == Self::REDIR_INPIPE.as_int()
    }
    pub const fn IS_REDIROP(self) -> bool {
        let s = self.as_int();
        s >= lextok::OUTANG.as_int() && s <= lextok::TRINANG.as_int()
    }
}
impl mnumber {
    /// mnumber is integer
    #[inline]
    pub const fn is_integer(self) -> bool {
        // self.type_ & MN_INTEGER != 0
        self.type_ == MN_INTEGER
    }
    /// mnumber is floating point
    #[inline]
    pub const fn is_float(self) -> bool {
        // self.type_ & MN_FLOAT != 0
        self.type_ == MN_FLOAT
    }
    /// mnumber not yet retrieved
    #[inline]
    pub const fn is_unset(self) -> bool {
        self.type_ & MN_UNSET != 0
    }
    pub const fn get_integer(self) -> Result<zlong, f64> {
        if !self.is_integer() {
            return Err(self.as_float());
        }
        Ok(self.as_integer())
    }
    pub const fn get_float(self) -> Result<f64, zlong> {
        if !self.is_float() {
            return Err(self.as_integer());
        }
        Ok(self.as_float())
    }

    /// Cast to an integer, regardless of the flags value
    ///
    /// # Safety
    ///
    /// It is currently safe to do this in Rust because of the existence of [`f64::to_bits`]
    #[inline(always)]
    pub const fn as_integer(self) -> zlong {
        unsafe { self.u.l }
    }
    /// Cast to a float, regardless of the flags value
    ///
    /// # Safety
    ///
    /// It is currently safe to do this in Rust because of the existence of [`f64::from_bits`]
    #[inline(always)]
    pub const fn as_float(self) -> f64 {
        unsafe { self.u.d }
    }
    /// `typeset -e`, create a new floating point numeric
    #[inline]
    pub const fn new_float(d: f64) -> Self {
        Self {
            u: mnumber__bindgen_ty_1 { d },
            type_: MN_FLOAT,
        }
    }
    /// `typeset -i`, create a new integer numeric
    #[inline]
    pub const fn new_integer(l: zlong) -> Self {
        Self {
            u: mnumber__bindgen_ty_1 { l },
            type_: MN_INTEGER,
        }
    }
    #[inline(always)]
    pub unsafe fn matheval(s: *mut c_char) -> Self {
        unsafe { matheval(s) }
    }
    pub const fn const_add(self, rhs: Self) -> Self {
        match (self.get_integer(), rhs.get_integer()) {
            (Ok(a), Ok(b)) => Self::new_integer(a + b),
            (Err(a), Err(b)) => Self::new_float(a + b),
            (Ok(a), Err(b)) => Self::new_float(a as f64 + b),
            (Err(a), Ok(b)) => Self::new_float(a + b as f64),
        }
    }
    pub const fn const_eq(self, other: Self) -> bool {
        match (self.get_integer(), other.get_integer()) {
            (Ok(a), Ok(b)) => a == b,
            (Err(a), Err(b)) => a == b,
            (Ok(a), Err(b)) => a == b as i64,
            (Err(a), Ok(b)) => a as i64 == b,
        }
    }
}
impl num_traits::Zero for mnumber {
    fn zero() -> Self {
        unsafe { zero_mnumber }
    }
    fn is_zero(&self) -> bool {
        /*
                static int
        notzero(mnumber a)
        {
            if ((a.type & MN_INTEGER) && a.u.l == 0) {
                zerr("division by zero");
                return 0;
            }
            return 1;
        }
                 */
        Self::zero().eq(self)
    }
    // fn set_zero(&mut self) {
    //     self.u = Self::zero().u;
    // }
}
impl Add for mnumber {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.const_add(rhs)
    }
}
impl PartialEq for mnumber {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.const_eq(*other)
    }
}
impl From<zlong> for mnumber {
    #[inline]
    fn from(value: zlong) -> Self {
        Self::new_integer(value)
    }
}
impl From<f64> for mnumber {
    #[inline]
    fn from(value: f64) -> Self {
        Self::new_float(value)
    }
}
impl TryFrom<mnumber> for zlong {
    type Error = f64;
    #[inline]
    fn try_from(value: mnumber) -> Result<Self, Self::Error> {
        value.get_integer()
    }
}
impl TryFrom<mnumber> for f64 {
    type Error = zlong;
    #[inline]
    fn try_from(value: mnumber) -> Result<Self, Self::Error> {
        value.get_float()
    }
}

impl mathfunc {
    #[inline]
    pub const fn NUMMATHFUNC(
        name: *mut c_char,
        func: NumMathFunc,
        min: c_int,
        max: c_int,
        id: c_int,
    ) -> Self {
        Self {
            next: null_mut(),
            name,
            flags: 0,
            nfunc: func,
            sfunc: None,
            module: null_mut(),
            minargs: min,
            maxargs: max,
            funcid: id,
        }
    }
    #[inline]
    pub const fn STRMATHFUNC(name: *mut c_char, func: StrMathFunc, id: c_int) -> Self {
        Self {
            next: null_mut(),
            name,
            flags: MFF_STR,
            nfunc: None,
            sfunc: func,
            module: null_mut(),
            minargs: 0,
            maxargs: 0,
            funcid: id,
        }
    }
}
impl QT {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
    pub const fn QT_IS_SINGLE(self) -> bool {
        let s = self.as_int();
        s == Self::QT_SINGLE.as_int() || s == Self::QT_SINGLE_OPTIONAL.as_int()
    }
}

impl linklist {
    /// Get the first node in the list, or `None` if is is null
    /// # Safety
    /// Caller asserts the pointer is not dangling or contended
    pub const unsafe fn firstnode(&self) -> Option<&linknode> {
        if self.first.is_null() {
            return None;
        }
        // SAFETY: We null-checked
        Some(unsafe { self.first.as_ref_unchecked() })
    }
}
impl linknode {
    /// Return the `next` link of the node.  Returns `null_mut()` if the
    /// node is the list terminator.
    pub const fn nextnode(&self) -> *mut linknode {
        self.next
    }

    /// Return the `prev` link of the node.
    pub const fn prevnode(&self) -> *mut linknode {
        self.prev
    }
}
impl hookdef {
    pub const fn HOOKDEF(name: *const c_char, func: Hookfn, flags: c_int) -> Self {
        Self {
            next: null_mut(),
            name,
            def: func,
            flags,
            funcs: null_mut(),
        }
    }
}

impl conddef {
    pub const fn CONDDEF(
        name: *mut c_char,
        flags: c_int,
        handler: CondHandler,
        min: c_int,
        max: c_int,
        condid: c_int,
    ) -> Self {
        Self {
            next: null_mut(),
            name,
            flags,
            handler,
            min,
            max,
            condid,
            module: null_mut(),
        }
    }
}
impl eprog {
    pub const fn is_real(&self) -> bool {
        self.flags & EF_REAL != 0
    }
    pub const fn is_heap(&self) -> bool {
        self.flags & EF_HEAP != 0
    }
    pub const fn is_map(&self) -> bool {
        self.flags & EF_MAP != 0
    }
    pub const fn is_run(&self) -> bool {
        self.flags & EF_RUN != 0
    }
    /// Shows whether the refcount is 0
    pub const fn should_delete(&self) -> bool {
        self.nref <= 0
    }
}

pub trait GsuTable {
    type Item;
    fn get(&mut self, p: *mut param) -> Self::Item;
    fn set(&mut self, p: *mut param, val: Self::Item);
    fn unset(&mut self, p: *mut param, idx: c_int);
}
macro_rules! gsutable {
    ($this:ty, $ty:ty) => {
        impl GsuTable for $this {
            type Item = $ty;
            fn get(&mut self, p: *mut param) -> Self::Item {
                let Some(f) = self.getfn else {
                    return Default::default();
                };
                unsafe { f(p) }
            }
            fn set(&mut self, p: *mut param, val: Self::Item) {
                if let Some(f) = self.setfn {
                    unsafe { f(p, val) }
                }
            }
            fn unset(&mut self, p: *mut param, idx: c_int) {
                if let Some(f) = self.unsetfn {
                    unsafe { f(p, idx) }
                }
            }
        }
    };
}
gsutable!(gsu_array, *mut *mut c_char);
gsutable!(gsu_float, f64);
gsutable!(gsu_hash, *mut hashtable);
gsutable!(gsu_integer, zlong);
gsutable!(gsu_scalar, *mut c_char);
impl GsuTable for c_void {
    type Item = ();
    fn get(&mut self, _p: *mut param) -> Self::Item {}
    fn set(&mut self, _p: *mut param, _val: Self::Item) {}
    fn unset(&mut self, _p: *mut param, _idx: c_int) {}
}

impl paramdef {
    #[inline(always)]
    pub const fn PM_TYPE(&self) -> c_int {
        PM_TYPE(self.flags)
    }
    /// Shorthand for common uses of adding parameters, with no special hash properties.
    #[inline]
    pub const fn PARAMDEF<T, G: GsuTable>(
        name: *mut c_char,
        flags: c_int,
        var: *mut T,
        gsu: *const G,
    ) -> Self {
        Self {
            name,
            flags,
            var: var.cast(),
            gsu: gsu.cast(),
            getnfn: None,
            scantfn: None,
            pm: null_mut(),
        }
    }
    /// Shorthand for common uses of adding parameters, with no special hash properties
    /// or gsu tables
    #[inline(always)]
    pub const fn PARAMDEF_NOGSU<T>(name: *mut c_char, flags: c_int, var: *mut T) -> Self {
        Self::PARAMDEF(name, flags, var, null::<c_void>())
    }
    /// Note that the following definitions are appropriate for defining
    /// parameters that reference a variable (var).  Hence the get/set/unset
    /// methods used will assume var needs dereferencing to get the value.
    #[inline(always)]
    pub const fn INTPARAMDEF(name: *mut c_char, var: *mut zlong) -> Self {
        Self::PARAMDEF_NOGSU(name, PM_INTEGER, var)
    }
    /// Note that the following definitions are appropriate for defining
    /// parameters that reference a variable (var).  Hence the get/set/unset
    /// methods used will assume var needs dereferencing to get the value.
    #[inline(always)]
    pub const fn STRPARAMDEF(name: *mut c_char, var: *mut c_char) -> Self {
        Self::PARAMDEF_NOGSU(name, PM_SCALAR, var)
    }
    /// Note that the following definitions are appropriate for defining
    /// parameters that reference a variable (var).  Hence the get/set/unset
    /// methods used will assume var needs dereferencing to get the value.
    #[inline(always)]
    pub const fn ARRPARAMDEF(name: *mut c_char, var: *mut c_void) -> Self {
        Self::PARAMDEF_NOGSU(name, PM_ARRAY, var)
    }
    /// The following is appropriate for a module function that behaves
    /// in a special fashion.  Parameters used in a module that don't
    /// have special behaviour shouldn't be declared in a table but
    /// should just be added with the standard parameter functions.
    ///
    /// These parameters are not marked as removable, since they
    /// shouldn't be loaded as local parameters, unlike the special
    /// Zle parameters that are added and removed on each call to Zle.
    /// We add the PM_REMOVABLE flag when removing the feature corresponding
    /// to the parameter.
    #[inline(always)]
    pub const fn SPECIALPMDEF<G: GsuTable>(
        name: *mut c_char,
        flags: c_int,
        gsufn: *const G,
        getnfn: GetNodeFunc,
        scantfn: ScanTabFunc,
    ) -> Self {
        // pub struct paramdef {
        //     pub name: *mut c_char,
        //     pub flags: c_int,
        //     pub var: *mut c_void,
        //     pub gsu: *const c_void,
        //     pub getnfn: GetNodeFunc,
        //     pub scantfn: ScanTabFunc,
        //     pub pm: *mut param,
        // }
        Self {
            name,
            flags: flags | PM_SPECIAL | PM_HIDE | PM_HIDEVAL,
            var: null_mut(),
            gsu: gsufn.cast(),
            getnfn,
            scantfn,
            pm: null_mut(),
        }
    }
    /// Check if this paramdef has been added to the parameter table.
    #[inline(always)]
    pub const fn is_linked(&self) -> bool {
        !self.pm.is_null()
    }
    /// Check if this paramdef's name matches a given C string.
    /// # Safety
    /// `other` must be a valid null-terminated C string.
    #[inline]
    pub unsafe fn name_matches(&self, other: &CStr) -> bool {
        unsafe { strcmp(self.name, other.as_ptr()) == 0 }
    }

    /// addparamdef
    #[inline(always)]
    pub unsafe fn add(&mut self) -> c_int {
        unsafe { addparamdef(&raw mut *self) }
    }
}

/// Extract the parameter type from flags.
/// Equivalent to `#define PM_TYPE(X) (X & (PM_SCALAR | PM_INTEGER | PM_EFLOAT | PM_FFLOAT | PM_ARRAY | PM_HASHED | PM_NAMEREF))`
#[inline]
pub const fn PM_TYPE(flags: c_int) -> c_int {
    const TYPE_MASK: c_int =
        PM_SCALAR | PM_INTEGER | PM_EFLOAT | PM_FFLOAT | PM_ARRAY | PM_HASHED | PM_NAMEREF;

    flags & TYPE_MASK
}
const PM_EFFLOAT: c_int = PM_EFLOAT | PM_FFLOAT;

impl param {
    /// Create a parameter, so that it can be assigned to.
    ///
    /// If a parameter of the same name exists in an outer scope, it is hidden
    /// by a newly created parameter.
    /// An already existing parameter node at the current level may be `created' and returned
    /// provided it is unset and not special.
    ///
    /// If the parameter can't be created because it already exists, the PM_UNSET flag is cleared.
    pub unsafe fn new(name: *mut c_char, flags: c_int) -> Option<&'static mut Self> {
        let param = unsafe { createparam(name, flags) };
        unsafe { param.as_mut() }
    }
    #[inline(always)]
    const fn flags_contains(&self, flag: c_int) -> bool {
        self.flags() & flag != 0
    }
    /// Get the parameter type (PM_SCALAR, PM_ARRAY, PM_INTEGER, etc.)
    #[inline]
    pub const fn PM_TYPE(&self) -> c_int {
        PM_TYPE(self.node.flags)
    }
    /// Is this a scalar (string) parameter?
    #[inline]
    pub const fn is_scalar(&self) -> bool {
        self.flags_contains(PM_SCALAR)
    }
    /// Is this an array parameter?
    #[inline]
    pub const fn is_array(&self) -> bool {
        self.flags_contains(PM_ARRAY)
    }
    /// Is this an integer parameter?
    #[inline]
    pub const fn is_integer(&self) -> bool {
        self.flags_contains(PM_INTEGER)
    }
    /// Is this a float parameter?
    #[inline]
    pub const fn is_float(&self) -> bool {
        self.flags_contains(PM_EFFLOAT)
    }
    /// Is this a hashed (association) parameter?
    #[inline]
    pub const fn is_hashed(&self) -> bool {
        self.flags_contains(PM_HASHED)
    }
    /// Is this a nameref parameter?
    #[inline]
    pub const fn is_nameref(&self) -> bool {
        self.flags_contains(PM_NAMEREF)
    }
    /// Is this a special builtin parameter?
    #[inline]
    pub const fn is_special(&self) -> bool {
        self.flags_contains(PM_SPECIAL)
    }
    /// Is this a readonly parameter?
    #[inline]
    pub const fn is_readonly(&self) -> bool {
        self.flags_contains(PM_READONLY)
    }
    /// Is this an exported parameter?
    #[inline]
    pub const fn is_exported(&self) -> bool {
        self.flags_contains(PM_EXPORTED)
    }
    /// Is this a local parameter?
    #[inline]
    pub const fn is_local(&self) -> bool {
        self.flags_contains(PM_LOCAL)
    }
    /// Has this parameter been unset?
    #[inline]
    pub const fn is_unset(&self) -> bool {
        self.flags_contains(PM_UNSET)
    }
    /// Is this parameter declared (explicitly named with typeset)?
    #[inline]
    pub const fn is_declared(&self) -> bool {
        self.flags_contains(PM_DECLARED)
    }
    /// Is this parameter tied to another?
    #[inline]
    pub const fn is_tied(&self) -> bool {
        self.flags_contains(PM_TIED)
    }
    /// Is this parameter restricted?
    #[inline]
    pub const fn is_restricted(&self) -> bool {
        self.flags_contains(PM_RESTRICTED)
    }
    /// Unset a parameter
    pub unsafe fn unset(&mut self) {
        unsafe { unsetparam_pm(&raw mut *self, 0, 1) };
    }
    pub unsafe fn reset(&mut self, flags: c_int) -> c_int {
        unsafe { resetparam(&raw mut *self, flags) }
    }
    /// Get the string value of this parameter.
    /// # Safety
    /// Caller must ensure this is a scalar parameter.
    #[inline]
    pub unsafe fn get_scalar(&self) -> *mut c_char {
        debug_assert!(self.is_scalar());

        unsafe {
            // SAFETY: Caller guarantees scalar type
            // gsu.s is ManuallyDrop<*const gsu_scalar>, need to dereference first
            // let sfn = (**self.gsu.s).get(p);
            let gsu_ptr = *self.gsu.s;
            if let Some(f) = (*gsu_ptr).getfn {
                // Get raw pointer from reference
                let this_ptr: *const param = self;
                f(this_ptr as *mut param)
            } else {
                null_mut()
            }
        }
    }
    /// Set the string value of this parameter.
    /// # Safety
    /// Caller must ensure this is a scalar parameter and value is valid.
    #[inline]
    pub unsafe fn set_scalar(&mut self, val: *mut c_char) {
        debug_assert!(self.is_scalar());

        unsafe {
            // SAFETY: Caller guarantees scalar type and valid value
            let gsu_ptr = *self.gsu.s;
            if let Some(f) = (*gsu_ptr).setfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param, val)
            }
        }
    }
    /// Get the integer value of this parameter.
    /// # Safety
    /// Caller must ensure this is an integer parameter.
    #[inline]
    pub unsafe fn get_int(&self) -> zlong {
        debug_assert!(self.is_integer());
        unsafe {
            // SAFETY: Caller guarantees integer type
            let gsu_ptr = *self.gsu.i;
            if let Some(f) = (*gsu_ptr).getfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param)
            } else {
                0
            }
        }
    }
    /// Set the integer value of this parameter.
    /// # Safety
    /// Caller must ensure this is an integer parameter.
    #[inline]
    pub unsafe fn set_int(&mut self, val: zlong) {
        debug_assert!(self.is_integer());
        unsafe {
            // SAFETY: Caller guarantees integer type
            let gsu_ptr = *self.gsu.i;
            if let Some(f) = (*gsu_ptr).setfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param, val)
            }
        }
    }
    /// Get the float value of this parameter.
    /// # Safety
    /// Caller must ensure this is a float parameter.
    #[inline]
    pub unsafe fn get_float(&self) -> f64 {
        debug_assert!(self.is_float());
        unsafe {
            // SAFETY: Caller guarantees float type
            let gsu_ptr = *self.gsu.f;
            if let Some(f) = (*gsu_ptr).getfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param)
            } else {
                0.0
            }
        }
    }
    /// Set the float value of this parameter.
    /// # Safety
    /// Caller must ensure this is a float parameter.
    #[inline]
    pub unsafe fn set_float(&mut self, val: f64) {
        debug_assert!(self.is_float());
        unsafe {
            // SAFETY: Caller guarantees float type
            let gsu_ptr = *self.gsu.f;
            if let Some(f) = (*gsu_ptr).setfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param, val)
            }
        }
    }
    /// Get the array value of this parameter.
    /// # Safety
    /// Caller must ensure this is an array parameter.
    #[inline]
    pub unsafe fn get_arr(&self) -> *mut *mut c_char {
        debug_assert!(self.is_array());
        unsafe {
            // SAFETY: Caller guarantees array type
            let gsu_ptr = *self.gsu.a;
            if let Some(f) = (*gsu_ptr).getfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param)
            } else {
                null_mut()
            }
        }
    }
    /// Set the array value of this parameter.
    /// # Safety
    /// Caller must ensure this is an array parameter.
    #[inline]
    pub unsafe fn set_arr(&mut self, val: *mut *mut c_char) {
        debug_assert!(self.is_array());
        unsafe {
            // SAFETY: Caller guarantees array type
            let gsu_ptr = *self.gsu.a;
            if let Some(f) = (*gsu_ptr).setfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param, val)
            }
        }
    }
    /// Get the hash table value of this parameter.
    /// # Safety
    /// Caller must ensure this is a hashed (association) parameter.
    #[inline]
    pub unsafe fn get_hash(&self) -> *mut hashtable {
        debug_assert!(self.is_hashed());
        unsafe {
            // SAFETY: Caller guarantees hashed type
            let gsu_ptr = *self.gsu.h;
            if let Some(f) = (*gsu_ptr).getfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param)
            } else {
                null_mut()
            }
        }
    }
    /// Set the hash table value of this parameter.
    /// # Safety
    /// Caller must ensure this is a hashed (association) parameter.
    #[inline]
    pub unsafe fn set_hash(&mut self, val: *mut hashtable) {
        debug_assert!(self.is_hashed());
        unsafe {
            // SAFETY: Caller guarantees hashed type
            let gsu_ptr = *self.gsu.h;
            if let Some(f) = (*gsu_ptr).setfn {
                let this_ptr: *const param = self;
                f(this_ptr as *mut param, val)
            }
        }
    }
    /// Get the parameter name. This may or may not be a zstr or zhstr
    #[inline]
    pub const fn name(&self) -> *mut c_char {
        self.node.nam
    }
    /// Get the parameter flags.
    #[inline]
    pub const fn flags(&self) -> c_int {
        self.node.flags
    }
    /// `typeset -x`, This param will be added to the environment of any subprocesses.
    #[inline]
    pub unsafe fn export(&mut self) {
        unsafe { export_param(&raw mut *self) }
    }
}

impl value {
    /// Get the parameter associated with this value.
    #[inline]
    pub const fn pm(&self) -> *mut param {
        self.pm
    }
    /// Is this an inverse subscript?
    #[inline]
    pub const fn is_inverse(&self) -> bool {
        self.valflags & (VALFLAG_INV as c_int) != 0
    }
    /// Is the subscripted range empty?
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.valflags & (VALFLAG_EMPTY as c_int) != 0
    }
    /// Is this a reference slice?
    #[inline]
    pub const fn is_refslice(&self) -> bool {
        self.valflags & (VALFLAG_REFSLICE as c_int) != 0
    }
    /// Is this a substitution (apply padding, case flags)?
    #[inline]
    pub const fn is_subst(&self) -> bool {
        self.valflags & (VALFLAG_SUBST as c_int) != 0
    }
    /// Get the stringified version of this. Notably this is tied to
    /// the current heap context.
    #[inline]
    pub unsafe fn to_zheap_str(&mut self) -> *mut c_char {
        unsafe { getstrvalue(&raw mut *self) }
    }
    /// Get the array version of this. Notably this is tied to
    /// the current heap context.
    #[inline]
    pub unsafe fn to_zheap_array(&mut self) -> *mut *mut c_char {
        unsafe { getarrvalue(&raw mut *self) }
    }
    #[inline]
    pub unsafe fn to_long(&mut self) -> zlong {
        unsafe { getintvalue(&raw mut *self) }
    }
    #[inline]
    pub unsafe fn to_number(&mut self) -> mnumber {
        unsafe { getnumvalue(&raw mut *self) }
    }

    #[inline]
    pub unsafe fn assign_str_value(&mut self, zstr: *mut c_char) {
        unsafe { setstrvalue(&raw mut *self, zstr) }
    }
    #[inline]
    pub unsafe fn assign_num_value(&mut self, num: impl Into<mnumber>) {
        let val = num.into();
        unsafe { setnumvalue(&raw mut *self, val) }
    }
}

/// A parameter type that we can lookup in zsh
pub trait ParamType: Sized {
    unsafe fn get_param_or_default(param: *mut c_char) -> Self;
    unsafe fn cast_value_or_default(value: &mut value) -> Self;
}
impl ParamType for zlong {
    unsafe fn cast_value_or_default(value: &mut value) -> Self {
        unsafe { value.to_long() }
    }
    unsafe fn get_param_or_default(param: *mut c_char) -> Self {
        unsafe { getiparam(param) }
    }
}
impl ParamType for MetaString {
    unsafe fn cast_value_or_default(value: &mut value) -> Self {
        Self {
            ptr: unsafe { value.to_zheap_str() },
        }
    }
    unsafe fn get_param_or_default(param: *mut c_char) -> Self {
        // unsafe { getsparam_u(param) }
        Self {
            ptr: unsafe { getsparam(param) },
        }
    }
}

/// Test for a shell emulation.  Use this rather than emulation directly.
#[inline]
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

/// `#define arena(X) ((char*)(X) + sizeof(struct heap))`
///
/// Creates a ptr WITHOUT provenance. This is a replica of the macro from C `zsh.h`
pub const fn arena(X: Heapid) -> *mut char {
    core::ptr::without_provenance_mut((X as usize) + size_of::<heap>())
}

// Those macros confuse me. This is less confusing.

/// Run a closure with a new temporary heap lasting as long as the closure.
/// # Safety
/// This function is *neither signal-safe, nor thread-safe*.
#[inline]
pub unsafe fn with_new_heap<R>(f: impl FnOnce() -> R) -> R {
    let old_heap = unsafe { new_heaps() };

    let res = f();

    unsafe { old_heaps(old_heap) };
    res
}
/// Run a closure with an arbitrary heap.
/// # Safety
/// This function is *neither signal-safe, nor thread-safe*.
#[inline]
pub unsafe fn with_heap<R>(h: *mut heap, f: impl FnOnce() -> R) -> R {
    let o = unsafe { switch_heaps(h) };

    let res = f();

    unsafe { switch_heaps(o) };
    res
}

pub unsafe fn IN_EVAL_TRAP() -> bool {
    unsafe { intrap != 0 && trapisfunc == 0 && traplocallevel == locallevel }
}

pub const unsafe fn EXITHOOK() -> &'static mut hookdef {
    unsafe { zshhooks.as_mut_ptr().as_mut_unchecked() }
}

pub const unsafe fn BEFORETRAPHOOK() -> &'static mut hookdef {
    unsafe { zshhooks.as_mut_ptr().add(1).as_mut_unchecked() }
}

pub const unsafe fn AFTERTRAPHOOK() -> &'static mut hookdef {
    unsafe { zshhooks.as_mut_ptr().add(2).as_mut_unchecked() }
}

pub const unsafe fn GETCOLORATTR() -> &'static mut hookdef {
    unsafe { zshhooks.as_mut_ptr().add(3).as_mut_unchecked() }
}

impl features {
    #[inline(always)]
    pub const fn new() -> Self {
        Self::CONST_DEFAULT
    }
    #[inline(always)]
    pub const fn builder() -> Self {
        Self::CONST_DEFAULT
    }
    pub const fn with_builtins(mut self, builtins: &'static mut [builtin]) -> Self {
        self.bn_list = builtins.as_mut_ptr();
        self.bn_size = builtins.len() as _;
        self
    }
    pub const fn with_params(mut self, params: &'static mut [paramdef]) -> Self {
        self.pd_list = params.as_mut_ptr();
        self.pd_size = params.len() as _;
        self
    }
    pub const fn with_conddefs(mut self, conddefs: &'static mut [conddef]) -> Self {
        self.cd_list = conddefs.as_mut_ptr();
        self.cd_size = conddefs.len() as _;
        self
    }
    pub const fn with_mathfuncs(mut self, mathfuncs: &'static mut [mathfunc]) -> Self {
        self.mf_list = mathfuncs.as_mut_ptr();
        self.mf_size = mathfuncs.len() as _;
        self
    }
    pub const fn with_n_abstract(mut self, n_abstract: c_int) -> Self {
        self.n_abstract = n_abstract;
        self
    }
    pub const CONST_DEFAULT: Self = Self {
        bn_list: null_mut(),
        bn_size: 0,
        cd_list: null_mut(),
        cd_size: 0,
        mf_list: null_mut(),
        mf_size: 0,
        pd_list: null_mut(),
        pd_size: 0,
        n_abstract: 0,
    };
}
impl Default for features {
    #[inline(always)]
    fn default() -> Self {
        Self::CONST_DEFAULT
    }
}

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

// #[inline]
// pub const fn HOOKDEF(name: *mut c_char, func: )

// SIGNALS

pub fn TRAPCOUNT() -> c_int {
    VSIGCOUNT + SIGRTMAX() - SIGRTMIN() + 1
}

pub fn SIGNUM(x: c_int) -> c_int {
    if x >= VSIGCOUNT {
        x - VSIGCOUNT + SIGRTMIN()
    } else {
        x
    }
}
pub fn SIGIDX(x: c_int) -> c_int {
    let min = SIGRTMIN();

    if x >= min && x <= SIGRTMAX() {
        x - min + VSIGCOUNT
    } else {
        x
    }
}
pub unsafe fn child_block() -> sigset_t {
    unsafe { signal_block(sigchld_mask) }
}
pub unsafe fn child_unblock() -> sigset_t {
    unsafe { signal_unblock(sigchld_mask) }
}
pub unsafe fn winch_block() -> sigset_t {
    unsafe { signal_block(signal_mask(SIGWINCH)) }
}
pub unsafe fn winch_unblock() -> sigset_t {
    unsafe { signal_unblock(signal_mask(SIGWINCH)) }
}
/// ignore a signal
pub unsafe fn signal_ignore(S: c_int) -> usize {
    unsafe { signal(S, SIG_IGN) }
}
/// return a signal to it default action
pub unsafe fn signal_default(S: c_int) -> usize {
    unsafe { signal(S, SIG_DFL) }
}

/// Use a circular queue to save signals caught during
/// critical sections of code.  You call queue_signals to
/// start queueing, and unqueue_signals to process the
/// queue and stop queueing.  Since the kernel doesn't
/// queue signals, it is probably overkill for zsh to do
/// this, but it shouldn't hurt anything to do it anyway.
pub unsafe fn run_queued_signals() {
    unsafe {
        while queue_front != queue_rear {
            queue_front = (queue_front + 1) % MAX_QUEUE_SIZE;
            let oset = signal_setmask(signal_mask_queue[queue_front as usize]);
            zhandler(signal_queue[queue_front as usize]);
            signal_setmask(oset);
        }
    }
}

unsafe fn queueing_enabled_offset<const OFFSET: c_int>() -> c_int {
    let old = unsafe { queueing_enabled };
    unsafe { queueing_enabled += OFFSET };
    old
}

/// `(queueing_enabled++)`
pub unsafe fn queue_signals() -> c_int {
    unsafe {
        queue_in += 1;
        queueing_enabled_offset::<1>()
    }
}
/// `if (!--queueing_enabled) run_queued_signals()`
pub unsafe fn unqueue_signals() {
    unsafe {
        queue_in -= 1;
        queueing_enabled -= 1;
        if queueing_enabled == 0 {
            run_queued_signals();
        }
    }
}
pub unsafe fn dont_queue_signals() {
    unsafe {
        queue_in = queueing_enabled;
        queueing_enabled = 0;
        run_queued_signals();
    }
}
pub unsafe fn restore_queued_signals(q: c_int) {
    unsafe {
        queue_in = q;
        queueing_enabled = q;
    }
}
pub unsafe fn queue_signal_level() -> c_int {
    unsafe { queueing_enabled }
}

// ZLE
//
// These are direct translations of public constants and hook convenience
// macros from Src/Zle/zle.h.  ZLE-private cursor/string macros such as INCCS,
// DECCS, ZS_memcpy, and ZC_iword depend on build-time multibyte choices and
// private ZLE symbols that are not currently exported by these bindings.

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

/*
#define WRAPDEF(func) \
    { NULL, 0, func, NULL }
/* Macros taking struct option * and char argument */
/* Option was set as -X */
#define OPT_MINUS(ops,c)	((ops)->ind[c] & 1)
/* Option was set as +X */
#define OPT_PLUS(ops,c)		((ops)->ind[c] & 2)
/*
 * Option was set any old how, maybe including an argument
 * (cheap test when we don't care).  Some bits of code
 * expect this to be 1 or 0.
 */
#define OPT_ISSET(ops,c)	((ops)->ind[c] != 0)
/* Option has an argument */
#define OPT_HASARG(ops,c)	((ops)->ind[c] > 3)
/* The argument for the option; not safe if it doesn't have one */
#define OPT_ARG(ops,c)		((ops)->args[((ops)->ind[c] >> 2) - 1])
/* Ditto, but safely returns NULL if there is no argument. */
#define OPT_ARG_SAFE(ops,c)	(OPT_HASARG(ops,c) ? OPT_ARG(ops,c) : NULL)

struct builtin {
    struct hashnode node;
    HandlerFunc handlerfunc;	/* pointer to function that executes this builtin     */
    int minargs;		/* minimum number of arguments                        */
    int maxargs;		/* maximum number of arguments, or -1 for no limit    */
    int funcid;			/* xbins (see above) for overloaded handlerfuncs      */
    char *optstr;		/* string of legal options (see execbuiltin())        */
    char *defopts;		/* options set by default for overloaded handlerfuncs */
};

#define BUILTIN(name, flags, handler, min, max, funcid, optstr, defopts) \
    { { NULL, name, flags }, handler, min, max, funcid, optstr, defopts }
#define BIN_PREFIX(name, flags) \
    BUILTIN(name, flags | BINF_PREFIX, NULLBINCMD, 0, 0, 0, NULL, NULL)

#define HOOKDEF(name, func, flags) { NULL, name, (Hookfn) func, flags, NULL }
/*
 * The following is appropriate for a module function that behaves
 * in a special fashion.  Parameters used in a module that don't
 * have special behaviour shouldn't be declared in a table but
 * should just be added with the standard parameter functions.
 *
 * These parameters are not marked as removable, since they
 * shouldn't be loaded as local parameters, unlike the special
 * Zle parameters that are added and removed on each call to Zle.
 * We add the PM_REMOVABLE flag when removing the feature corresponding
 * to the parameter.
 */
#define SPECIALPMDEF(name, flags, gsufn, getfn, scanfn) \
    { name, flags | PM_SPECIAL | PM_HIDE | PM_HIDEVAL, \
        NULL, gsufn, getfn, scanfn, NULL }

        /* Parts of the code where history expansion is disabled *
 * should be within a pair of STOPHIST ... ALLOWHIST     */



*/
#[inline]
pub fn STOPHIST() {
    unsafe { stophist += 4 }
}
#[inline]
pub fn ALLOWHIST() {
    unsafe { stophist -= 4 }
}
