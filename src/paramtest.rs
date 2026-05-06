use alloc::{format, string::String, vec::Vec};
use core::ffi::{CStr, c_char};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::printing::stdout_println;
use zsh_sys::macros::{ParamError, ParamKind, ParamRef};
use zsh_sys::{PM_SCALAR, mnumber, zalloc, zlong, ztrdup};

const SCALAR: *mut c_char = c"VLK_PARAMTEST_SCALAR".as_ptr().cast_mut();
const INTEGER: *mut c_char = c"VLK_PARAMTEST_INTEGER".as_ptr().cast_mut();
const FLOAT: *mut c_char = c"VLK_PARAMTEST_FLOAT".as_ptr().cast_mut();
const ARRAY: *mut c_char = c"VLK_PARAMTEST_ARRAY".as_ptr().cast_mut();
const HASH: *mut c_char = c"VLK_PARAMTEST_HASH".as_ptr().cast_mut();
const CREATED: *mut c_char = c"VLK_PARAMTEST_CREATED".as_ptr().cast_mut();
const MISSING: *mut c_char = c"VLK_PARAMTEST_MISSING".as_ptr().cast_mut();
const EXCOUNT: *mut c_char = c"EXCOUNT".as_ptr().cast_mut();

const ALL_TEST_PARAMS: [*mut c_char; 6] = [SCALAR, INTEGER, FLOAT, ARRAY, HASH, CREATED];

fn show_kind(kind: ParamKind) -> &'static str {
    match kind {
        ParamKind::Scalar => "scalar",
        ParamKind::Array => "array",
        ParamKind::Integer => "integer",
        ParamKind::Float => "float",
        ParamKind::Hash => "hash",
        ParamKind::Nameref => "nameref",
        ParamKind::Other(_) => "other",
    }
}

fn show_error(err: ParamError) -> &'static str {
    match err {
        ParamError::NullName => "NullName",
        ParamError::NullParam => "NullParam",
        ParamError::NotFound => "NotFound",
        ParamError::Hidden => "Hidden",
        ParamError::ReadOnly => "ReadOnly",
        ParamError::ZshRejected => "ZshRejected",
        ParamError::BadKind => "BadKind",
        ParamError::TypeMismatch { .. } => "TypeMismatch",
        ParamError::NullGsu => "NullGsu",
        ParamError::MissingGet => "MissingGet",
        ParamError::MissingSet => "MissingSet",
        ParamError::MissingUnset => "MissingUnset",
    }
}

unsafe fn zsh_dup(s: *const c_char) -> *mut c_char {
    unsafe { ztrdup(s) }
}

unsafe fn zsh_char_array(items: &[*const c_char]) -> *mut *mut c_char {
    let len = items.len() + 1;
    let bytes = len * size_of::<*mut c_char>();
    let out = unsafe { zalloc(bytes) }.cast::<*mut c_char>();
    if out.is_null() {
        return null_mut();
    }

    for (idx, item) in items.iter().enumerate() {
        unsafe { *out.add(idx) = zsh_dup(*item) };
    }
    unsafe { *out.add(items.len()) = null_mut() };
    out
}

unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::from("<null>");
    }
    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => String::from(s),
        Err(_) => String::from("<non-utf8>"),
    }
}

unsafe fn cstr_eq(ptr: *const c_char, expected: &str) -> bool {
    !ptr.is_null() && unsafe { CStr::from_ptr(ptr) }.to_bytes() == expected.as_bytes()
}

unsafe fn argv_strings(mut ptr: *mut *mut c_char, max_items: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = 0;
    while !ptr.is_null() && seen < max_items {
        let item = unsafe { *ptr };
        if item.is_null() {
            break;
        }
        out.push(unsafe { cstr_to_string(item) });
        seen += 1;
        ptr = unsafe { ptr.add(1) };
    }
    out
}

fn pass(label: &str) {
    stdout_println(&format!("  ok   {label}"));
}

fn fail(label: &str, failures: &mut usize, detail: &str) {
    *failures += 1;
    stdout_println(&format!("  FAIL {label}: {detail}"));
}

fn expect(label: &str, condition: bool, failures: &mut usize, detail: &str) {
    if condition {
        pass(label);
    } else {
        fail(label, failures, detail);
    }
}

fn expect_ok<T>(label: &str, result: Result<T, ParamError>, failures: &mut usize) -> Option<T> {
    match result {
        Ok(value) => {
            pass(label);
            Some(value)
        }
        Err(err) => {
            fail(label, failures, show_error(err));
            None
        }
    }
}

unsafe fn cleanup_test_params() {
    for name in ALL_TEST_PARAMS {
        let _ = unsafe { ParamRef::unset_named(name) };
    }
}

/// Run a broad smoke/regression test for `zsh_sys::macros::ParamRef`.
///
/// It deliberately uses zsh's own allocation (`ztrdup`/`zalloc`) for values that
/// setter APIs take ownership of.  Handles that may be invalidated by zsh are
/// dropped and looked up again by stable C-string names.
pub unsafe fn run_parameter_tests() -> i32 {
    stdout_println("[paramtest] starting zsh parameter wrapper checks");
    let mut failures = 0usize;

    unsafe { cleanup_test_params() };

    expect(
        "missing lookup returns None",
        unsafe { ParamRef::lookup_direct(MISSING, true).is_none() },
        &mut failures,
        "unexpected parameter exists",
    );

    match unsafe { ParamRef::set_scalar_named(null_mut(), null_mut()) } {
        Err(ParamError::NullName) => pass("null name is rejected"),
        Ok(_) => fail("null name is rejected", &mut failures, "unexpected success"),
        Err(err) => fail("null name is rejected", &mut failures, show_error(err)),
    }

    if let Some(created) = expect_ok(
        "create scalar with createparam",
        unsafe { ParamRef::create(CREATED, PM_SCALAR) },
        &mut failures,
    ) {
        let kind = unsafe { created.kind() };
        expect(
            "created kind is scalar",
            kind == ParamKind::Scalar,
            &mut failures,
            show_kind(kind),
        );
        let _ = unsafe { created.unset() };
    }

    if let Some(scalar) = expect_ok(
        "set scalar by name",
        unsafe { ParamRef::set_scalar_named(SCALAR, zsh_dup(c"alpha".as_ptr())) },
        &mut failures,
    ) {
        let kind = unsafe { scalar.kind() };
        expect(
            "scalar kind",
            kind == ParamKind::Scalar,
            &mut failures,
            show_kind(kind),
        );
        match unsafe { scalar.get_scalar() } {
            Ok(value) => expect(
                "scalar get current",
                unsafe { cstr_eq(value, "alpha") },
                &mut failures,
                &unsafe { cstr_to_string(value) },
            ),
            Err(err) => fail("scalar get current", &mut failures, show_error(err)),
        }
        match unsafe { scalar.get_integer() } {
            Err(ParamError::TypeMismatch { .. }) => pass("scalar/integer mismatch checked"),
            Ok(value) => fail(
                "scalar/integer mismatch checked",
                &mut failures,
                &format!("unexpected integer {value}"),
            ),
            Err(err) => fail(
                "scalar/integer mismatch checked",
                &mut failures,
                show_error(err),
            ),
        }
        if let Some(scalar) = expect_ok(
            "scalar current set",
            unsafe { scalar.assign_scalar(zsh_dup(c"beta".as_ptr())) },
            &mut failures,
        ) {
            match unsafe { scalar.get_scalar() } {
                Ok(value) => expect(
                    "scalar current set observed",
                    unsafe { cstr_eq(value, "beta") },
                    &mut failures,
                    &unsafe { cstr_to_string(value) },
                ),
                Err(err) => fail(
                    "scalar current set observed",
                    &mut failures,
                    show_error(err),
                ),
            }
        }
    }

    if let Some(integer) = expect_ok(
        "set integer by name",
        unsafe { ParamRef::set_integer_named(INTEGER, 41 as zlong) },
        &mut failures,
    ) {
        expect(
            "integer kind",
            unsafe { integer.kind() } == ParamKind::Integer,
            &mut failures,
            "not integer",
        );
        match unsafe { integer.get_integer() } {
            Ok(value) => expect(
                "integer get current",
                value == 41,
                &mut failures,
                &format!("got {value}"),
            ),
            Err(err) => fail("integer get current", &mut failures, show_error(err)),
        }
        if let Some(integer) = expect_ok(
            "integer current set",
            unsafe { integer.assign_integer(42 as zlong) },
            &mut failures,
        ) {
            match unsafe { integer.get_integer() } {
                Ok(value) => expect(
                    "integer current set observed",
                    value == 42,
                    &mut failures,
                    &format!("got {value}"),
                ),
                Err(err) => fail(
                    "integer current set observed",
                    &mut failures,
                    show_error(err),
                ),
            }
        }
    }

    if let Some(float_pm) = expect_ok(
        "set float by name",
        unsafe { ParamRef::set_number_named(FLOAT, mnumber::new_float(2.5)) },
        &mut failures,
    ) {
        expect(
            "float kind",
            unsafe { float_pm.kind() } == ParamKind::Float,
            &mut failures,
            "not float",
        );
        match unsafe { float_pm.get_float() } {
            Ok(value) => expect(
                "float get current",
                (value - 2.5).abs() < 0.000_001,
                &mut failures,
                &format!("got {value}"),
            ),
            Err(err) => fail("float get current", &mut failures, show_error(err)),
        }
        if let Some(float_pm) = expect_ok(
            "float current set",
            unsafe { float_pm.assign_float(3.25) },
            &mut failures,
        ) {
            match unsafe { float_pm.get_float() } {
                Ok(value) => expect(
                    "float current set observed",
                    (value - 3.25).abs() < 0.000_001,
                    &mut failures,
                    &format!("got {value}"),
                ),
                Err(err) => fail("float current set observed", &mut failures, show_error(err)),
            }
        }
    }

    if let Some(array_pm) = expect_ok(
        "set array by name",
        unsafe {
            ParamRef::set_array_named(
                ARRAY,
                zsh_char_array(&[c"red".as_ptr(), c"green".as_ptr(), c"blue".as_ptr()]),
            )
        },
        &mut failures,
    ) {
        expect(
            "array kind",
            unsafe { array_pm.kind() } == ParamKind::Array,
            &mut failures,
            "not array",
        );
        match unsafe { array_pm.get_array() } {
            Ok(values) => {
                let values = unsafe { argv_strings(values, 8) };
                expect(
                    "array values",
                    values == ["red", "green", "blue"],
                    &mut failures,
                    &format!("got {values:?}"),
                );
            }
            Err(err) => fail("array values", &mut failures, show_error(err)),
        }
    }

    if let Some(hash_pm) = expect_ok(
        "set hash by name",
        unsafe {
            ParamRef::set_hash_pairs_named(
                HASH,
                zsh_char_array(&[
                    c"alpha".as_ptr(),
                    c"one".as_ptr(),
                    c"beta".as_ptr(),
                    c"two".as_ptr(),
                ]),
            )
        },
        &mut failures,
    ) {
        expect(
            "hash kind",
            unsafe { hash_pm.kind() } == ParamKind::Hash,
            &mut failures,
            "not hash",
        );
        match unsafe { ParamRef::get_hash_pairs_named(HASH) } {
            Ok(values) => {
                let values = unsafe { argv_strings(values, 16) };
                expect(
                    "hash values contain expected values",
                    values.iter().any(|v| v == "one") && values.iter().any(|v| v == "two"),
                    &mut failures,
                    &format!("got {values:?}"),
                );
            }
            Err(err) => fail(
                "hash values contain expected values",
                &mut failures,
                show_error(err),
            ),
        }
        match unsafe { ParamRef::get_hash_keys_named(HASH) } {
            Ok(keys) => {
                let keys = unsafe { argv_strings(keys, 16) };
                expect(
                    "hash keys contain expected keys",
                    keys.iter().any(|v| v == "alpha") && keys.iter().any(|v| v == "beta"),
                    &mut failures,
                    &format!("got {keys:?}"),
                );
            }
            Err(err) => fail(
                "hash keys contain expected keys",
                &mut failures,
                show_error(err),
            ),
        }
        match unsafe { hash_pm.get_hash_table() } {
            Ok(table) => expect(
                "hash table pointer non-null",
                !table.is_null(),
                &mut failures,
                "null table",
            ),
            Err(err) => fail(
                "hash table pointer non-null",
                &mut failures,
                show_error(err),
            ),
        }
    }

    if let Some(excount) = unsafe { ParamRef::lookup_direct(EXCOUNT, true) } {
        let kind = unsafe { excount.kind() };
        expect(
            "EXCOUNT special parameter is integer",
            kind == ParamKind::Integer,
            &mut failures,
            show_kind(kind),
        );
        match unsafe { excount.get_integer() } {
            Ok(value) => pass(&format!("EXCOUNT read through GSU ({value})")),
            Err(err) => fail("EXCOUNT read through GSU", &mut failures, show_error(err)),
        }
    } else {
        fail("EXCOUNT lookup", &mut failures, "not found");
    }

    if let Some(pm) = unsafe { ParamRef::lookup_direct(SCALAR, true) } {
        match unsafe { pm.reset_type(ParamKind::Integer) } {
            Ok(()) => pass("reset scalar to integer"),
            Err(err) => fail("reset scalar to integer", &mut failures, show_error(err)),
        }
        match unsafe { ParamRef::lookup_direct(SCALAR, true) } {
            Some(pm) => expect(
                "reset kind observed",
                unsafe { pm.kind() } == ParamKind::Integer,
                &mut failures,
                "not integer after reset",
            ),
            None => fail(
                "reset kind observed",
                &mut failures,
                "parameter disappeared",
            ),
        }
    } else {
        fail("reset scalar to integer", &mut failures, "scalar missing");
    }

    for name in [SCALAR, INTEGER, FLOAT, ARRAY, HASH] {
        match unsafe { ParamRef::unset_checked_named(name) } {
            Ok(()) => pass(&format!("unset {}", unsafe { cstr_to_string(name) })),
            Err(err) => fail(
                &format!("unset {}", unsafe { cstr_to_string(name) }),
                &mut failures,
                show_error(err),
            ),
        }
    }

    expect(
        "cleanup removed scalar",
        unsafe { ParamRef::lookup_direct(SCALAR, true).is_none() },
        &mut failures,
        "still present",
    );

    if failures == 0 {
        stdout_println("[paramtest] all parameter checks passed");
        0
    } else {
        stdout_println(&format!("[paramtest] {failures} parameter check(s) failed"));
        1
    }
}
