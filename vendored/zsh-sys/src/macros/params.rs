#[allow(unused_imports)]
use super::*;
use core::ptr::NonNull;
use {super::strings::MetaString, ::bytemuck::TransparentWrapper, ::core::mem::ManuallyDrop};
use {crate::*, ::core::fmt::Display};

/// A valid member of the GSU union. Useful if your module adds a new variable type
/// # Safety
/// Caller asserts zsh parameters can take this as a valid GSU
pub unsafe trait GsuTable {
    type Item;
    /// a bitor "any-of" bitflag union
    const PM_TYPEFLAG_UNION: c_int;
    fn get(&self, p: *mut param) -> Option<Self::Item>;
    fn set(&self, p: *mut param, val: Self::Item);
    fn unset(&self, p: *mut param, idx: c_int);
}
macro_rules! gsutable {
    (%nn $this:ty, $ty:ty, $flag:expr) => {
        unsafe impl GsuTable for $this {
            type Item = NonNull<$ty>;
            const PM_TYPEFLAG_UNION: c_int = $flag;
            fn get(&self, p: *mut param) -> Option<Self::Item> {
                let f = self.getfn?;
                NonNull::new(unsafe { f(p) })
            }
            fn set(&self, p: *mut param, val: Self::Item) {
                if let Some(f) = self.setfn {
                    unsafe { f(p, val.as_ptr()) }
                }
            }
            fn unset(&self, p: *mut param, idx: c_int) {
                if let Some(f) = self.unsetfn {
                    unsafe { f(p, idx) }
                }
            }
        }
    };
    (%val $this:ty, $ty:ty, $flag:expr) => {
        unsafe impl GsuTable for $this {
            type Item = $ty;
            const PM_TYPEFLAG_UNION: c_int = $flag;
            fn get(&self, p: *mut param) -> Option<Self::Item> {
                let f = self.getfn?;
                Some(unsafe { f(p) })
            }
            fn set(&self, p: *mut param, val: Self::Item) {
                if let Some(f) = self.setfn {
                    unsafe { f(p, val) }
                }
            }
            fn unset(&self, p: *mut param, idx: c_int) {
                if let Some(f) = self.unsetfn {
                    unsafe { f(p, idx) }
                }
            }
        }
    };
}
gsutable!(%nn gsu_array, *mut c_char, PM_ARRAY);
gsutable!(%val gsu_float, f64, PM_EFLOAT | PM_FFLOAT);
gsutable!(%nn gsu_hash, hashtable, PM_HASHED);
gsutable!(%val gsu_integer, zlong, PM_INTEGER);
gsutable!(%nn gsu_scalar, c_char, PM_SCALAR);

// SAFETY: None lmao, I'm just doing this so our APIs don't suck
unsafe impl GsuTable for c_void {
    type Item = ();
    const PM_TYPEFLAG_UNION: c_int = PM_TYPE_MASK;
    fn get(&self, _p: *mut param) -> Option<Self::Item> {
        Some(())
    }
    fn set(&self, _p: *mut param, _val: Self::Item) {}
    fn unset(&self, _p: *mut param, _idx: c_int) {}
}
impl param__bindgen_ty_2 {
    pub const fn is_null(&self) -> bool {
        // SAFETY: Every single field of this union is a pointer type
        ManuallyDrop::into_inner(unsafe { self.s }).is_null()
    }
    /// Get the field of the union that corresponds to the table type
    #[inline(always)]
    pub const fn get_table_for_type<G: GsuTable>(&self) -> *const G {
        // Always use const-eval branches here so it monomorphizes into literally nothing
        // SAFETY: Validity guaranteed by the trait implementors
        if G::PM_TYPEFLAG_UNION == gsu_array::PM_TYPEFLAG_UNION {
            ManuallyDrop::into_inner(unsafe { self.a }).cast()
        } else if G::PM_TYPEFLAG_UNION == gsu_float::PM_TYPEFLAG_UNION {
            ManuallyDrop::into_inner(unsafe { self.f }).cast()
        } else if G::PM_TYPEFLAG_UNION == gsu_hash::PM_TYPEFLAG_UNION {
            ManuallyDrop::into_inner(unsafe { self.h }).cast()
        } else if G::PM_TYPEFLAG_UNION == gsu_integer::PM_TYPEFLAG_UNION {
            ManuallyDrop::into_inner(unsafe { self.i }).cast()
        } else if G::PM_TYPEFLAG_UNION == gsu_scalar::PM_TYPEFLAG_UNION {
            ManuallyDrop::into_inner(unsafe { self.s }).cast()
        } else {
            null()
        }
    }
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

const PM_TYPE_MASK: c_int =
    PM_SCALAR | PM_INTEGER | PM_EFLOAT | PM_FFLOAT | PM_ARRAY | PM_HASHED | PM_NAMEREF;

/// Extract the parameter type from flags.
/// Equivalent to `#define PM_TYPE(X) (X & (PM_SCALAR | PM_INTEGER | PM_EFLOAT | PM_FFLOAT | PM_ARRAY | PM_HASHED | PM_NAMEREF))`
#[inline]
pub const fn PM_TYPE(flags: c_int) -> c_int {
    flags & PM_TYPE_MASK
}
impl param {
    /// Create a parameter, so that it can be assigned to.
    ///
    /// If a parameter of the same name exists in an outer scope, it is hidden
    /// by a newly created parameter.
    /// An already existing parameter node at the current level may be `created' and returned
    /// provided it is unset and not special.
    ///
    /// If the parameter can't be created because it already exists, the PM_UNSET flag is cleared.
    pub unsafe fn new(name: *mut c_char, flags: c_int) -> Option<NonNull<Self>> {
        let param = unsafe { createparam(name, flags) };
        NonNull::new(param)
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
    /// Returns `true` if the inner type equals the desired table type
    pub const fn has_type<G: GsuTable>(&self) -> bool {
        self.PM_TYPE() == G::PM_TYPEFLAG_UNION
    }
    /// Is this a nameref parameter?
    #[inline]
    pub const fn is_nameref(&self) -> bool {
        self.PM_TYPE() == PM_NAMEREF
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
    /// Is this special parameter allowed to be removed from `paramtab`?
    #[inline]
    pub const fn is_removable(&self) -> bool {
        self.flags_contains(PM_REMOVABLE)
    }

    pub unsafe fn unset(&mut self) {
        unsafe { unsetparam_pm(&raw mut *self, 0, 1) };
    }
    pub unsafe fn reset(&mut self, flags: c_int) -> c_int {
        unsafe { resetparam(&raw mut *self, flags) }
    }

    /// Deref the inner GSU (As long as it is one of the common types!).
    /// Returns `None` if it is unable to deref
    ///
    /// # Safety
    /// Caller asserts the ptr is not dangling
    #[inline(always)]
    const unsafe fn deref_standard_gsu<G: GsuTable>(&self) -> Option<&G> {
        // Common sense: The derefs below quite literally cannot be valid if we
        // aren't the correct type!
        if !self.has_type::<G>() {
            return None;
        }

        let nnp: *const G = self.gsu.get_table_for_type();
        if nnp.is_null() {
            return None;
        }
        // I just return a ref here foy my own convenience more than anything
        // SAFETY: We assert the requested table kind is valid, and it is checked at
        // runtime by `get_table_for_type`
        unsafe { nnp.as_ref() }
    }
    /// inner GSU table get
    /// # Safety
    /// This memory is managed by zsh internally, do not free it.
    /// Caller asserts this struct is pinned and they called this with the right type
    pub unsafe fn standard_gsu_get<G: GsuTable>(&mut self) -> Option<G::Item> {
        let selfptr = &raw mut *self;
        unsafe { self.deref_standard_gsu::<G>() }?.get(selfptr)
    }
    /// inner GSU table set
    /// # Safety
    /// This memory is managed by zsh internally, do not free it.
    /// Caller asserts this struct is pinned and they called this with the right type
    pub unsafe fn standard_gsu_set<G: GsuTable>(&mut self, val: G::Item) -> Result<(), G::Item> {
        let selfptr = &raw mut *self;
        let ptab = unsafe { self.deref_standard_gsu::<G>() };

        match ptab {
            Some(g) => {
                g.set(selfptr, val);
                Ok(())
            }
            None => Err(val),
        }
    }
    /// inner GSU table unset. This is the proper way to free vars.
    /// # Safety
    /// This memory is managed by zsh internally, do not free it.
    /// Caller asserts this struct is pinned and they called this with the right type
    pub unsafe fn standard_gsu_unset<G: GsuTable>(&mut self, idx: c_int) -> Result<(), c_int> {
        let selfptr = &raw mut *self;
        let ptab = unsafe { self.deref_standard_gsu::<G>() }.ok_or(idx)?;
        ptab.unset(selfptr, idx);
        Ok(())
    }

    /// Get the parameter name. This can be freed by calling `zsfree` iirc?
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

/// The zsh parameter type bits, normalized so `PM_SCALAR == 0` is explicit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParamKind {
    Scalar,
    Array,
    Integer,
    Float,
    Hash,
    Nameref,
    Other(c_int),
}

impl ParamKind {
    #[inline]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Scalar => "scalar parameter",
            Self::Array => "array parameter",
            Self::Integer => "integer parameter",
            Self::Float => "float parameter",
            Self::Hash => "hash parameter",
            Self::Nameref => "nameref parameter",
            Self::Other(..) => "unknown parameter kind",
        }
    }

    #[inline]
    pub const fn str(self) -> &'static str {
        self.message()
    }

    #[inline]
    pub const fn from_flags(flags: c_int) -> Self {
        match PM_TYPE(flags) {
            PM_SCALAR => Self::Scalar,
            PM_ARRAY => Self::Array,
            PM_INTEGER => Self::Integer,
            PM_EFLOAT | PM_FFLOAT => Self::Float,
            PM_HASHED => Self::Hash,
            PM_NAMEREF => Self::Nameref,
            other => Self::Other(other),
        }
    }

    #[inline]
    pub const fn reset_flags(self) -> Option<c_int> {
        match self {
            Self::Scalar => Some(PM_SCALAR),
            Self::Array => Some(PM_ARRAY),
            Self::Integer => Some(PM_INTEGER),
            Self::Float => Some(PM_FFLOAT),
            Self::Hash => Some(PM_HASHED),
            Self::Nameref => Some(PM_NAMEREF),
            Self::Other(_) => None,
        }
    }
}
impl Display for ParamKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let sstr = self.str();
        match self {
            Self::Other(i) => write!(f, "{sstr}: {i}"),
            _ => sstr.fmt(f),
        }
    }
}

/// Checked failure modes for the safe-ish parameter wrapper.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParamError {
    NullName,
    NullParam,
    NotFound,
    Hidden,
    ReadOnly,
    ZshRejected,
    BadKind,
    TypeMismatch {
        expected: ParamKind,
        found: ParamKind,
    },
    NullGsu,
    MissingGet,
    MissingSet,
    MissingUnset,
}
impl ParamError {
    #[inline]
    pub const fn message(self) -> &'static str {
        match self {
            Self::NullName => "parameter name is null",
            Self::NullParam => "parameter pointer is null",
            Self::NotFound => "parameter was not found",
            Self::Hidden => "parameter is hidden by another visible parameter",
            Self::ReadOnly => "parameter is read-only",
            Self::ZshRejected => "zsh rejected the parameter operation",
            Self::BadKind => "parameter kind cannot be used for this operation",
            Self::TypeMismatch { .. } => "parameter type mismatch",
            Self::NullGsu => "parameter GSU table is null",
            Self::MissingGet => "parameter GSU table has no get function",
            Self::MissingSet => "parameter GSU table has no set function",
            Self::MissingUnset => "parameter GSU table has no unset function",
        }
    }

    #[inline]
    pub const fn str(self) -> &'static str {
        self.message()
    }
}
impl Display for ParamError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let sstr = self.str();
        match self {
            Self::TypeMismatch { expected, found } => {
                write!(f, "{sstr}, expected {expected}, found {found}")
            }
            _ => sstr.fmt(f),
        }
    }
}
impl core::error::Error for ParamError {}

/// A non-null handle to a zsh `Param`.
///
/// This is only "safe-ish": zsh can still invalidate a `Param` through other
/// shell activity, scope teardown, `unset`, or type changes.  The wrapper avoids
/// the easy segfaults by checking null pointers, checking the current parameter
/// kind before touching the get/set/unset table, and routing type-changing
/// assignment through zsh's own parameter entry points.
///
/// Methods that may remove or replace the parameter take `self` by value to make
/// the invalidation visible at the Rust call site.
#[derive(Debug, TransparentWrapper, Copy, Clone)]
#[repr(transparent)]
pub struct ParamRef {
    ptr: NonNull<param>,
}

impl ParamRef {
    /// Wrap a raw `Param` pointer if it is non-null.
    ///
    /// # Safety
    /// The pointer must either be null or point at a live zsh `struct param`.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut param) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Return the raw `Param` pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut param {
        self.ptr.as_ptr()
    }

    /// Borrow the underlying `param`.
    ///
    /// # Safety
    /// The caller must ensure zsh has not invalidated this handle.
    #[inline]
    pub unsafe fn as_param(&self) -> &param {
        unsafe { self.ptr.as_ref() }
    }

    /// Mutably borrow the underlying `param`.
    ///
    /// # Safety
    /// The caller must ensure zsh has not invalidated this handle and there is
    /// no concurrent or re-entrant access to the same parameter.
    #[inline]
    pub unsafe fn as_param_mut(&mut self) -> &mut param {
        unsafe { self.ptr.as_mut() }
    }

    /// Look up a visible parameter through the current `paramtab`.
    /// Avoids autoloading if `direct == true`.
    ///
    /// # Regular lookup
    /// This uses zsh's table `getnode` hook, so it follows the same visible
    /// lookup rules as C code using `paramtab->getnode(paramtab, name)`.
    ///
    /// # Direct lookup
    /// This mirrors zsh's common `gethashnode2(paramtab, name)` direct-table
    /// access when `paramtab == realparamtab`, and otherwise falls back to the
    /// table's ordinary `getnode` hook.
    ///
    /// # Safety
    /// `name` must be a valid nul-terminated zsh string.
    #[inline]
    pub unsafe fn lookup_direct(name: *const c_char, direct: bool) -> Option<Self> {
        let pt = NonNull::new(unsafe { paramtab })?;
        unsafe { Self::lookup_in(pt, name, direct) }
    }

    unsafe fn lookup_in(
        tab: NonNull<hashtable>,
        name: *const c_char,
        direct: bool,
    ) -> Option<Self> {
        if name.is_null() {
            return None;
        }

        let node = if direct && tab.addr().get() == unsafe { realparamtab }.addr() {
            unsafe { gethashnode2(tab.as_ptr(), name) }
        } else if direct {
            let get = unsafe { tab.as_ref().getnode2.or(tab.as_ref().getnode) }?;
            unsafe { get(tab.as_ptr(), name) }
        } else {
            let get = unsafe { tab.as_ref().getnode }?;
            unsafe { get(tab.as_ptr(), name) }
        };

        unsafe { Self::from_raw(node.cast()) }
    }

    /// Create a parameter with zsh's `createparam()`.
    ///
    /// # Safety
    /// `name` must be a valid mutable nul-terminated zsh string.  zsh may reject
    /// readonly, hidden, nameref, or otherwise invalid creations by returning
    /// null.
    #[inline]
    pub unsafe fn create(name: *mut c_char, flags: c_int) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_raw(createparam(name, flags)).ok_or(ParamError::ZshRejected) }
    }

    #[inline]
    pub unsafe fn name(&self) -> *mut c_char {
        unsafe { self.as_param() }.name()
    }

    #[inline]
    pub unsafe fn flags(&self) -> c_int {
        unsafe { self.as_param() }.flags()
    }

    #[inline]
    pub unsafe fn kind(&self) -> ParamKind {
        ParamKind::from_flags(unsafe { self.as_param() }.flags())
    }

    #[inline]
    pub unsafe fn is_unset(&self) -> bool {
        unsafe { self.as_param() }.is_unset()
    }

    #[inline]
    pub unsafe fn is_readonly(&self) -> bool {
        unsafe { self.as_param() }.is_readonly()
    }

    #[inline]
    pub unsafe fn is_special(&self) -> bool {
        unsafe { self.as_param() }.is_special()
    }

    #[inline]
    pub unsafe fn is_removable(&self) -> bool {
        unsafe { self.as_param() }.is_removable()
    }

    #[inline]
    unsafe fn ensure_kind(&self, expected: ParamKind) -> Result<(), ParamError> {
        let found = unsafe { self.kind() };
        if found == expected {
            Ok(())
        } else {
            Err(ParamError::TypeMismatch { expected, found })
        }
    }

    #[inline]
    unsafe fn ensure_writable(&self) -> Result<(), ParamError> {
        let pm = unsafe { self.as_param() };
        if pm.is_readonly() && pm.level <= unsafe { locallevel } {
            Err(ParamError::ReadOnly)
        } else {
            Ok(())
        }
    }

    #[inline]
    unsafe fn ensure_can_call_unset(&self) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let pm = unsafe { self.as_param() };
        if !pm.is_unset() || pm.is_removable() {
            let gsu = unsafe { *pm.gsu.s };
            if gsu.is_null() {
                return Err(ParamError::NullGsu);
            }
            if unsafe { (*gsu).unsetfn }.is_none() {
                return Err(ParamError::MissingUnset);
            }
        }
        Ok(())
    }

    #[inline]
    unsafe fn scalar_gsu(&self) -> Result<*const gsu_scalar, ParamError> {
        unsafe { self.ensure_kind(ParamKind::Scalar)? };
        let gsu = unsafe { *self.as_param().gsu.s };
        if gsu.is_null() {
            Err(ParamError::NullGsu)
        } else {
            Ok(gsu)
        }
    }

    #[inline]
    unsafe fn integer_gsu(&self) -> Result<*const gsu_integer, ParamError> {
        unsafe { self.ensure_kind(ParamKind::Integer)? };
        let gsu = unsafe { *self.as_param().gsu.i };
        if gsu.is_null() {
            Err(ParamError::NullGsu)
        } else {
            Ok(gsu)
        }
    }

    #[inline]
    unsafe fn float_gsu(&self) -> Result<*const gsu_float, ParamError> {
        unsafe { self.ensure_kind(ParamKind::Float)? };
        let gsu = unsafe { *self.as_param().gsu.f };
        if gsu.is_null() {
            Err(ParamError::NullGsu)
        } else {
            Ok(gsu)
        }
    }

    #[inline]
    unsafe fn array_gsu(&self) -> Result<*const gsu_array, ParamError> {
        unsafe { self.ensure_kind(ParamKind::Array)? };
        let gsu = unsafe { *self.as_param().gsu.a };
        if gsu.is_null() {
            Err(ParamError::NullGsu)
        } else {
            Ok(gsu)
        }
    }

    #[inline]
    unsafe fn hash_gsu(&self) -> Result<*const gsu_hash, ParamError> {
        unsafe { self.ensure_kind(ParamKind::Hash)? };
        let gsu = unsafe { *self.as_param().gsu.h };
        if gsu.is_null() {
            Err(ParamError::NullGsu)
        } else {
            Ok(gsu)
        }
    }

    /// Get the current scalar value through this parameter's GSU table.
    #[inline]
    pub unsafe fn get_scalar(&self) -> Result<*mut c_char, ParamError> {
        let gsu = unsafe { self.scalar_gsu()? };
        let get = unsafe { (*gsu).getfn }.ok_or(ParamError::MissingGet)?;
        Ok(unsafe { get(self.as_ptr()) })
    }

    /// Set the current scalar value through this parameter's GSU table.
    ///
    /// `val` follows the same ownership rules as the underlying zsh set function:
    /// for ordinary parameters zsh takes ownership and may `zsfree()` it later.
    #[inline]
    pub unsafe fn set_scalar_current(&self, val: *mut c_char) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let gsu = unsafe { self.scalar_gsu()? };
        let set = unsafe { (*gsu).setfn }.ok_or(ParamError::MissingSet)?;

        unsafe { set(self.as_ptr(), val) };
        Ok(())
    }

    #[inline]
    pub unsafe fn get_integer(&self) -> Result<zlong, ParamError> {
        let gsu = unsafe { self.integer_gsu()? };
        let get = unsafe { (*gsu).getfn }.ok_or(ParamError::MissingGet)?;
        Ok(unsafe { get(self.as_ptr()) })
    }

    #[inline]
    pub unsafe fn set_integer_current(&self, val: zlong) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let gsu = unsafe { self.integer_gsu()? };
        let set = unsafe { (*gsu).setfn }.ok_or(ParamError::MissingSet)?;

        unsafe { set(self.as_ptr(), val) };
        Ok(())
    }

    #[inline]
    pub unsafe fn get_float(&self) -> Result<f64, ParamError> {
        let gsu = unsafe { self.float_gsu()? };
        let get = unsafe { (*gsu).getfn }.ok_or(ParamError::MissingGet)?;
        Ok(unsafe { get(self.as_ptr()) })
    }

    #[inline]
    pub unsafe fn set_float_current(&self, val: f64) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let gsu = unsafe { self.float_gsu()? };
        let set = unsafe { (*gsu).setfn }.ok_or(ParamError::MissingSet)?;

        unsafe { set(self.as_ptr(), val) };
        Ok(())
    }

    #[inline]
    pub unsafe fn get_array(&self) -> Result<*mut *mut c_char, ParamError> {
        let gsu = unsafe { self.array_gsu()? };
        let get = unsafe { (*gsu).getfn }.ok_or(ParamError::MissingGet)?;
        Ok(unsafe { get(self.as_ptr()) })
    }

    /// Set the current array value through this parameter's GSU table.
    ///
    /// `val` must be a null-terminated zsh-owned array with zsh-owned elements.
    #[inline]
    pub unsafe fn set_array_current(&self, val: *mut *mut c_char) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let gsu = unsafe { self.array_gsu()? };
        let set = unsafe { (*gsu).setfn }.ok_or(ParamError::MissingSet)?;

        unsafe { set(self.as_ptr(), val) };
        Ok(())
    }

    #[inline]
    pub unsafe fn get_hash_table(&self) -> Result<*mut hashtable, ParamError> {
        let gsu = unsafe { self.hash_gsu()? };
        let get = unsafe { (*gsu).getfn }.ok_or(ParamError::MissingGet)?;
        Ok(unsafe { get(self.as_ptr()) })
    }

    #[inline]
    pub unsafe fn set_hash_table_current(&self, val: *mut hashtable) -> Result<(), ParamError> {
        unsafe { self.ensure_writable()? };
        let gsu = unsafe { self.hash_gsu()? };
        let set = unsafe { (*gsu).setfn }.ok_or(ParamError::MissingSet)?;

        unsafe { set(self.as_ptr(), val) };
        Ok(())
    }

    /// Assign the current scalar value without changing this parameter's type.
    ///
    /// For type-changing assignment, use `set_scalar_named()` with a stable name
    /// pointer (not `pm->node.nam`) and then look the parameter up again.
    #[inline]
    pub unsafe fn assign_scalar(self, val: *mut c_char) -> Result<Self, ParamError> {
        unsafe { self.set_scalar_current(val)? };
        Ok(self)
    }

    /// Assign the current integer value without changing this parameter's type.
    #[inline]
    pub unsafe fn assign_integer(self, val: zlong) -> Result<Self, ParamError> {
        unsafe { self.set_integer_current(val)? };
        Ok(self)
    }

    /// Assign the current numeric value without changing this parameter's type.
    #[inline]
    pub unsafe fn assign_number(self, val: mnumber) -> Result<Self, ParamError> {
        match unsafe { self.kind() } {
            ParamKind::Integer => {
                let val = match val.get_integer() {
                    Ok(val) => val,
                    Err(val) => val as zlong,
                };
                unsafe { self.set_integer_current(val)? };
            }
            ParamKind::Float => {
                let val = match val.get_float() {
                    Ok(val) => val,
                    Err(val) => val as f64,
                };
                unsafe { self.set_float_current(val)? };
            }
            found => {
                return Err(ParamError::TypeMismatch {
                    expected: ParamKind::Integer,
                    found,
                });
            }
        }
        Ok(self)
    }

    /// Assign the current float value without changing this parameter's type.
    #[inline]
    pub unsafe fn assign_float(self, val: f64) -> Result<Self, ParamError> {
        unsafe { self.set_float_current(val)? };
        Ok(self)
    }

    /// Assign the current array value without changing this parameter's type.
    ///
    /// zsh takes ownership of `val` and its elements.
    #[inline]
    pub unsafe fn assign_array(self, val: *mut *mut c_char) -> Result<Self, ParamError> {
        unsafe { self.set_array_current(val)? };
        Ok(self)
    }

    /// Assign the current hash table without changing this parameter's type.
    #[inline]
    pub unsafe fn assign_hash_table(self, val: *mut hashtable) -> Result<Self, ParamError> {
        unsafe { self.set_hash_table_current(val)? };
        Ok(self)
    }

    /// Reset this parameter's type using zsh's `resetparam()`.
    ///
    /// This consumes the handle because `resetparam()` unsets the old parameter
    /// and creates a replacement.  Look it up by name again before further use.
    #[inline]
    pub unsafe fn reset_type(self, kind: ParamKind) -> Result<(), ParamError> {
        let flags = kind.reset_flags().ok_or(ParamError::BadKind)?;
        unsafe { self.reset_flags(flags) }
    }

    /// Reset this parameter's type to an explicit zsh flag set.
    ///
    /// This mirrors zsh's `resetparam()` logic without calling that symbol: some
    /// zsh builds declare it in headers but do not export it for modules.  The
    /// parameter name is copied to a fixed stack buffer before `unsetparam_pm()`
    /// invalidates this handle; names longer than the buffer are rejected.
    #[inline]
    pub unsafe fn reset_flags(self, flags: c_int) -> Result<(), ParamError> {
        unsafe { self.ensure_can_call_unset()? };

        let name = unsafe { self.name() };
        if name.is_null() {
            return Err(ParamError::NullName);
        }

        if let Some(visible) = unsafe { Self::lookup_direct(name, true) }
            && visible.as_ptr() != self.as_ptr()
        {
            return Err(ParamError::Hidden);
        }

        let mut name_buf = [0 as c_char; 256];
        let mut idx = 0usize;
        while idx + 1 < name_buf.len() {
            let ch = unsafe { *name.add(idx) };
            name_buf[idx] = ch;
            if ch == 0 {
                break;
            }
            idx += 1;
        }
        if name_buf[idx] != 0 {
            return Err(ParamError::ZshRejected);
        }

        let rc = unsafe { unsetparam_pm(self.as_ptr(), 0, 1) };
        if rc != 0 {
            return Err(ParamError::ZshRejected);
        }

        unsafe { Self::from_raw(createparam(name_buf.as_mut_ptr(), flags)) }
            .map(|_| ())
            .ok_or(ParamError::ZshRejected)
    }

    /// Unset this parameter using `unsetparam_pm()`.
    ///
    /// This may free the parameter node, so the handle is consumed.
    #[inline]
    pub unsafe fn unset(self) -> Result<(), ParamError> {
        unsafe { self.ensure_can_call_unset()? };
        let rc = unsafe { unsetparam_pm(self.as_ptr(), 0, 1) };
        if rc == 0 {
            Ok(())
        } else {
            Err(ParamError::ZshRejected)
        }
    }

    #[inline]
    unsafe fn from_assignment(ptr: *mut param) -> Result<Self, ParamError> {
        unsafe { Self::from_raw(ptr).ok_or(ParamError::ZshRejected) }
    }

    #[inline]
    pub unsafe fn get_integer_named(name: *mut c_char) -> Result<zlong, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { getiparam(name) })
    }

    #[inline]
    pub unsafe fn get_number_named(name: *mut c_char) -> Result<mnumber, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { getnparam(name) })
    }

    #[inline]
    pub unsafe fn get_scalar_named(name: *mut c_char) -> Result<*mut c_char, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { getsparam(name) })
    }

    #[inline]
    pub unsafe fn get_array_named(name: *mut c_char) -> Result<*mut *mut c_char, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { getaparam(name) })
    }

    #[inline]
    pub unsafe fn get_hash_pairs_named(name: *mut c_char) -> Result<*mut *mut c_char, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { gethparam(name) })
    }

    #[inline]
    pub unsafe fn get_hash_keys_named(name: *mut c_char) -> Result<*mut *mut c_char, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        Ok(unsafe { gethkparam(name) })
    }

    #[inline]
    pub unsafe fn set_scalar_named(
        name: *mut c_char,
        val: *mut c_char,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(setsparam(name, val)) }
    }

    #[inline]
    pub unsafe fn assign_scalar_named(
        name: *mut c_char,
        val: *mut c_char,
        flags: c_int,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(assignsparam(name, val, flags)) }
    }

    #[inline]
    pub unsafe fn set_integer_named(name: *mut c_char, val: zlong) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(setiparam(name, val)) }
    }

    #[inline]
    pub unsafe fn assign_integer_named(
        name: *mut c_char,
        val: zlong,
        flags: c_int,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(assigniparam(name, val, flags)) }
    }

    #[inline]
    pub unsafe fn set_integer_no_convert_named(
        name: *mut c_char,
        val: zlong,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(setiparam_no_convert(name, val)) }
    }

    #[inline]
    pub unsafe fn set_number_named(name: *mut c_char, val: mnumber) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(setnparam(name, val)) }
    }

    #[inline]
    pub unsafe fn set_float_named(name: *mut c_char, val: f64) -> Result<Self, ParamError> {
        unsafe { Self::set_number_named(name, mnumber::new_float(val)) }
    }

    #[inline]
    pub unsafe fn set_array_named(
        name: *mut c_char,
        val: *mut *mut c_char,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(setaparam(name, val)) }
    }

    #[inline]
    pub unsafe fn assign_array_named(
        name: *mut c_char,
        val: *mut *mut c_char,
        flags: c_int,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(assignaparam(name, val, flags)) }
    }

    #[inline]
    pub unsafe fn set_hash_pairs_named(
        name: *mut c_char,
        val: *mut *mut c_char,
    ) -> Result<Self, ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { Self::from_assignment(sethparam(name, val)) }
    }

    /// Reset a named parameter after looking it up directly.
    ///
    /// Unlike `reset_type(self, ...)`, this keeps the caller's stable name
    /// pointer outside the invalidated `Param` node.
    #[inline]
    pub unsafe fn reset_named(name: *mut c_char, kind: ParamKind) -> Result<(), ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        let pm = unsafe { Self::lookup_direct(name, true) }.ok_or(ParamError::NotFound)?;
        unsafe { pm.reset_type(kind) }
    }

    /// Unset by name using zsh's `unsetparam()`.
    ///
    /// This function mirrors the C helper, which returns no status.
    #[inline]
    pub unsafe fn unset_named(name: *mut c_char) -> Result<(), ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        unsafe { unsetparam(name) };
        Ok(())
    }

    /// Unset a named parameter after looking it up directly, preserving an error
    /// return from `unsetparam_pm()`.
    #[inline]
    pub unsafe fn unset_checked_named(name: *mut c_char) -> Result<(), ParamError> {
        if name.is_null() {
            return Err(ParamError::NullName);
        }
        let pm = unsafe { Self::lookup_direct(name, true) }.ok_or(ParamError::NotFound)?;
        unsafe { pm.unset() }
    }
}
