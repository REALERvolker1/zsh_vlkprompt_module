use super::*;

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
