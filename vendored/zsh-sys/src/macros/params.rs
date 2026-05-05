use super::strings::MetaString;
use super::*;

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
