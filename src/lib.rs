//! Zsh example module in Rust using FFI
//!
//! This module provides the same functionality as the original C example module:
//! - A builtin command `example`
//! - Parameter definitions for integer, string, and array
//! - Conditions for pattern length and string matching
//! - Math functions for sum and length
//! - Function wrapper for global dot behavior

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr;
use libc::{__errno_location, size_t, ssize_t, strerror};

// Bitflags for zsh options
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ZshOptions: u32 {
        const GLOBDOTS = 0x01;
        const INTERACTIVE = 0x02;
        // Add more as needed
    }
}

// Zsh types
#[repr(C)]
pub struct Param {
    // This would normally be more complex
    pub name: *const c_char,
    pub flags: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct Options {
    pub ind: [c_char; 128],
    pub args: *mut *mut c_char,
    pub argscount: c_int,
    pub argsalloc: c_int,
}

#[repr(C)]
pub struct Builtin {
    pub name: *const c_char,
    pub handler: extern "C" fn(*const c_char, *mut *mut c_char, *mut Options, c_int) -> c_int,
    pub minargs: c_int,
    pub maxargs: c_int,
    pub funcid: c_int,
    pub optstr: *const c_char,
    pub defopts: *const c_char,
}

#[repr(C)]
pub struct Conddef {
    pub name: *const c_char,
    pub flags: c_int,
    pub handler: extern "C" fn(*mut *mut c_char, c_int) -> c_int,
    pub min: c_int,
    pub max: c_int,
    pub condid: c_int,
    pub module: *const c_char,
}

#[repr(C)]
pub struct MathFunc {
    pub next: *mut MathFunc,
    pub name: *const c_char,
    pub flags: c_int,
    pub nfunc: extern "C" fn(*const c_char, c_int, *mut Mnumber, c_int) -> Mnumber,
    pub sfunc: extern "C" fn(*const c_char, *const c_char, c_int) -> Mnumber,
    pub module: *const c_char,
    pub minargs: c_int,
    pub maxargs: c_int,
    pub funcid: c_int,
}
#[repr(C)]
pub struct Mnumber {
    pub u: MnumberUnion,
    pub type_: c_int,
}

#[repr(C)]
pub union MnumberUnion {
    pub l: c_long,
    pub d: f64,
}

#[repr(C)]
pub struct Paramdef {
    pub name: *const c_char,
    pub flags: c_int,
    pub var: *mut c_void,
    pub gsu: *const c_void,
    pub getnfn: *const c_void,
    pub scantfn: *const c_void,
    pub pm: *mut Param,
}

#[repr(C)]
pub struct FuncWrap {
    pub next: *mut FuncWrap,
    pub flags: c_int,
    pub handler: extern "C" fn(*mut c_void, *mut FuncWrap, *const c_char) -> c_int,
    pub module: *mut c_void,
}
#[repr(C)]
pub struct Module {
    pub node: *mut c_void, // HashNode
    pub u: ModuleUnion,
    pub autoloads: *mut c_void, // LinkList
    pub deps: *mut c_void,      // LinkList
    pub wrapper: c_int,
}

#[repr(C)]
pub union ModuleUnion {
    pub handle: *mut c_void,
    pub linked: *mut c_void, // Linkedmod
    pub alias: *const c_char,
}

// External functions from zsh (FFI declarations)
unsafe extern "C" {
    fn zstrdup(s: *const c_char) -> *mut c_char;
    fn zsfree(s: *mut c_char);
    fn zarrdup(arr: *mut *mut c_char) -> *mut *mut c_char;
    fn freearray(arr: *mut *mut c_char);
    fn ztrlen(s: *const c_char) -> size_t;
    fn dyncat(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    // fn strlen(s: *const c_char) -> size_t;
    // fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    // fn printf(fmt: *const c_char, ...) -> c_int;
    // fn fflush(stream: *mut c_void) -> c_int;
    // fn errno() -> *mut c_int;
    // fn strerror(errnum: c_int) -> *const c_char;
    fn addwrapper(m: *mut Module, w: *mut FuncWrap) -> c_int;
    fn deletewrapper(m: *mut Module, w: *mut FuncWrap) -> c_int;
    fn setfeatureenables(m: *mut Module, features: *mut c_void, enables: *mut c_int) -> c_int;
    fn featuresarray(m: *mut Module, features: *mut c_void) -> *mut *mut c_char;
    fn handlefeatures(m: *mut Module, features: *mut c_void, enables: *mut *mut c_int) -> c_int;
}
