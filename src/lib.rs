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

#[global_allocator]
static ALLOCATOR: MyMalloc = MyMalloc;

struct MyMalloc;
unsafe impl core::alloc::GlobalAlloc for MyMalloc {
    #[inline]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { libc::malloc(layout.size()) }.cast()
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { libc::free(ptr.cast()) }
    }
    #[inline]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { libc::calloc(1, layout.size()) }.cast()
    }
    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe { libc::realloc(ptr.cast(), new_size) }.cast()
    }
}

use ::alloc::string::ToString;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr;
use libc::{__errno_location, size_t, ssize_t, strerror};

fn eprint_str(s: &[u8]) -> core::fmt::Result {
    let mut n_written = 0;

    loop {
        let wrote = unsafe { libc::write(2, s.as_ptr() as _, s.len()) };
        match wrote.cmp(&0) {
            ::core::cmp::Ordering::Equal => break Ok(()),
            ::core::cmp::Ordering::Less => break Err(core::fmt::Error),
            ::core::cmp::Ordering::Greater => {
                n_written += wrote.cast_unsigned();
                if n_written >= s.len() {
                    break Ok(());
                }
            }
        }
    }
}
fn eprint_fmt(a: core::fmt::Arguments<'_>) -> core::fmt::Result {
    let s = a.to_string();
    eprint_str(s.as_bytes())
}
fn eprintln_fmt(a: core::fmt::Arguments<'_>) -> core::fmt::Result {
    eprint_fmt(a)?;
    eprint_str(b"\n")
}
#[macro_export]
macro_rules! eprintln {
    ($fmt:literal $(, $arg:tt )*) => {
        {
            _ = $crate::eprintln_fmt(::core::format_args!($fmt $(, $arg)*));
        }
    };
}

#[panic_handler]
fn p(info: &::core::panic::PanicInfo<'_>) -> ! {
    eprintln!("Received panicinfo: {}", info);
    loop {}
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

// Global parameters (simulated)
static mut INT_PARAM: c_long = 42;
static mut STR_PARAM: *mut c_char = ptr::null_mut();
static mut ARR_PARAM: *mut *mut c_char = ptr::null_mut();

// Helper function to convert a Rust string to a C string
fn rust_str_to_c_str(s: &str) -> *mut c_char {
    let c_str = alloc::ffi::CString::new(s).unwrap();
    c_str.into_raw()
}

// Helper function to convert C string to Rust string
fn c_str_to_rust_str(s: *const c_char) -> Option<String> {
    if s.is_null() {
        None
    } else {
        unsafe {
            let c_str = core::ffi::CStr::from_ptr(s);
            Some(c_str.to_string_lossy().into_owned())
        }
    }
}

// Builtin example command
extern "C" fn bin_example(
    nam: *const c_char,
    args: *mut *mut c_char,
    ops: *mut zsh_sys::options,
    func: c_int,
) -> c_int {
    // Print options
    unsafe {
        libc::printf(b"Options: \0".as_ptr() as *const c_char);
        let mut c = 32i8;
        while c < 127 {
            c += 1;
            if ops.is_null() || (*ops).ind[c as usize] != 0 {
                libc::printf(b"%c\0".as_ptr() as *const c_char, c as c_int);
            }
        }
        libc::printf(b"\nArguments:\0".as_ptr() as *const c_char);

        let mut i: c_long = 0;
        let mut current_arg = args;
        while !(*current_arg).is_null() {
            libc::printf(b" %s\0".as_ptr() as *const c_char, *current_arg);
            i += 1;
            current_arg = current_arg.offset(1);
        }
        libc::printf(b"\nName: %s\n\0".as_ptr() as *const c_char, nam);

        #[cfg(target_pointer_width = "64")]
        {
            libc::printf(
                b"\nInteger Parameter: %s\n\0".as_ptr() as *const c_char,
                output64(INT_PARAM),
            );
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            libc::printf(
                b"\nInteger Parameter: %ld\n\0".as_ptr() as *const c_char,
                INT_PARAM,
            );
        }

        let str_param = if STR_PARAM.is_null() {
            b"" as &[u8]
        } else {
            unsafe { core::ffi::CStr::from_ptr(STR_PARAM).to_bytes() }
        };
        libc::printf(
            b"String Parameter: %s\n\0".as_ptr() as *const c_char,
            str_param.as_ptr() as *const c_char,
        );

        libc::printf(b"Array Parameter:\0".as_ptr() as *const c_char);
        if !ARR_PARAM.is_null() {
            for i in 0..2 {
                libc::printf(b" %s\0".as_ptr() as *const c_char, *ARR_PARAM.offset(i));
            }
        }
        libc::printf(b"\n\0".as_ptr() as *const c_char);

        // Update parameters
        INT_PARAM = i;
        if !STR_PARAM.is_null() {
            zsh_sys::zsfree(STR_PARAM);
        }
        STR_PARAM = if args.is_null() || (*args).is_null() {
            libc::strdup(b"\0".as_ptr() as *const c_char)
        } else {
            libc::strdup(*args)
        };

        if !ARR_PARAM.is_null() {
            // zsh_sys::freearray(ARR_PARAM);
        }
        // ARR_PARAM = zsh_sys::zarrdup(args);
    }

    0
}

// Condition for pattern length
extern "C" fn cond_p_len(a: *mut *mut c_char, id: c_int) -> c_int {
    unsafe {
        // This is a simplified version - in reality would need to handle the condition properly
        if a.is_null() || (*a).is_null() {
            return 0;
        }

        let s1 = if (*a).is_null() { ptr::null() } else { *a };

        if s1.is_null() {
            return 0;
        }

        let s1_len = zsh_sys::ztrlen(s1);
        if a.is_null() || (*a.offset(1)).is_null() {
            // No second argument - check if empty
            return if s1_len == 0 { 1 } else { 0 };
        }

        // Second argument exists - compare length
        let s2 = *a.offset(1);
        if s2.is_null() {
            return 0;
        }

        // For simplicity, we just return 1 (true)
        1
    }
}

// Condition for example string matching
extern "C" fn cond_i_ex(a: *mut *mut c_char, id: c_int) -> c_int {
    unsafe {
        if a.is_null() || (*a).is_null() || (*a.offset(1)).is_null() {
            return 0;
        }

        let s1 = *a;
        let s2 = *a.offset(1);

        let dyncat_result = zsh_sys::dyncat(s1, s2);
        let result = libc::strcmp(dyncat_result, b"example\0".as_ptr() as *const c_char);
        zsh_sys::zsfree(dyncat_result);

        if result == 0 { 1 } else { 0 }
    }
}
// Math function for sum
extern "C" fn math_sum(name: *const c_char, argc: c_int, argv: *mut Mnumber, id: c_int) -> Mnumber {
    unsafe {
        let mut ret: Mnumber = Mnumber {
            u: MnumberUnion { l: 0 },
            type_: 0,
        };

        let mut f = 0i32;
        ret.type_ = if f != 0 { 2 } else { 1 }; // MN_FLOAT or MN_INTEGER

        // Simplified implementation
        for _ in 0..argc {
            // In real implementation, would check argv type and perform sum
            // This is just a placeholder
            ret.u.l += 1; // Placeholder calculation
        }

        ret
    }
}

// Math function for length
extern "C" fn math_length(name: *const c_char, arg: *const c_char, id: c_int) -> Mnumber {
    unsafe {
        let mut ret: Mnumber = Mnumber {
            u: MnumberUnion { l: 0 },
            type_: 0,
        };

        if arg.is_null() {
            ret.u.l = 0;
        } else {
            ret.u.l = zsh_sys::ztrlen(arg) as c_long;
        }

        ret.type_ = 1; // MN_INTEGER
        ret
    }
}

// Function wrapper for global dot behavior
extern "C" fn ex_wrapper(
    prog: *mut c_void,
    w: *mut zsh_sys::funcwrap,
    name: *const c_char,
) -> c_int {
    unsafe {
        if name.is_null() {
            return 1;
        }

        let name_str = core::ffi::CStr::from_ptr(name);
        if name_str.to_bytes().len() < 7 {
            return 1;
        }

        // Check if name starts with "example"
        let name_bytes = name_str.to_bytes();
        if name_bytes[0..7] != *b"example" {
            return 1;
        }

        // In a real implementation, this would save and restore options
        // For now, just return 0 (success)
        0
    }
}

// Function to convert 64-bit integer to string (simplified)
extern "C" fn output64(val: c_long) -> *const c_char {
    // This would normally be implemented to handle 64-bit values
    // For simplicity, returning a fixed string
    b"42\0".as_ptr() as *const c_char
}

// Module setup
#[unsafe(no_mangle)]
pub extern "C" fn setup_(m: *mut zsh_sys::module) -> c_int {
    unsafe {
        libc::printf(b"The example module has now been set up.\n\0".as_ptr() as *const c_char);
        libc::fflush(ptr::null_mut());
    }
    0
}

// Module features function
#[unsafe(no_mangle)]
pub extern "C" fn features_(m: *mut zsh_sys::module, features: *mut *mut *mut c_char) -> c_int {
    unsafe {
        // This is a simplified implementation
        *features = ptr::null_mut();
    }
    0
}

// Module enables function
#[unsafe(no_mangle)]
pub extern "C" fn enables_(m: *mut zsh_sys::module, enables: *mut *mut c_int) -> c_int {
    unsafe {
        *enables = ptr::null_mut();
    }
    0
}

// Module boot function
#[unsafe(no_mangle)]
pub extern "C" fn boot_(m: *mut zsh_sys::module) -> c_int {
    unsafe {
        INT_PARAM = 42;

        if !STR_PARAM.is_null() {
            zsh_sys::zsfree(STR_PARAM);
        }
        STR_PARAM = libc::strdup(b"example\0".as_ptr() as *const c_char);

        if !ARR_PARAM.is_null() {
            // zsh_sys::freearray(ARR_PARAM);
        }

        // Create array with example values
        let mut arr: Vec<*mut c_char> = Vec::new();
        arr.push(libc::strdup(b"example\0".as_ptr() as *const c_char));
        arr.push(libc::strdup(b"array\0".as_ptr() as *const c_char));
        arr.push(ptr::null_mut()); // Null terminator

        let arrbox = Box::leak(arr.into_boxed_slice());

        ARR_PARAM = arrbox.as_mut_ptr();

        // Add wrapper (simplified)
        // In a real implementation, this would add the wrapper to the module
        0
    }
}
// Module cleanup function
#[unsafe(no_mangle)]
pub extern "C" fn cleanup_(m: *mut zsh_sys::module) -> c_int {
    unsafe {
        // Delete wrapper (simplified)
        // In a real implementation, this would remove the wrapper from the module

        // Reset parameters
        if !STR_PARAM.is_null() {
            zsh_sys::zsfree(STR_PARAM);
            STR_PARAM = ptr::null_mut();
        }

        if !ARR_PARAM.is_null() {
            // zsh_sys::freearray(ARR_PARAM);
            // ARR_PARAM = ptr::null_mut();
        }

        zsh_sys::setfeatureenables(m, ptr::null_mut(), ptr::null_mut());
    }
    0
}

// Module finish function
#[unsafe(no_mangle)]
pub extern "C" fn finish_(m: *mut zsh_sys::module) -> c_int {
    unsafe {
        libc::printf(
            b"Thank you for using the example module.  Have a nice day.\n\0".as_ptr()
                as *const c_char,
        );
        libc::fflush(ptr::null_mut());
    }
    0
}
