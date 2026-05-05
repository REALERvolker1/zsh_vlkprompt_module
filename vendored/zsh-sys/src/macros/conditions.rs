#[allow(unused_imports)]
use super::*;
use crate::*;

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
