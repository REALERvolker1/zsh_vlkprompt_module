use super::*;

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
