use super::*;

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
