use super::*;

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
