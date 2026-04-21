use ::bindgen::callbacks::ParseCallbacks;
use ::std::io::Write;
use std::env;
use std::path::PathBuf;

// If your build is failing, please, take a look at config.h and change its values accordingly to
// your machine.
// compile_error!("todo");

struct VecWriter(pub Vec<u8>);
impl Write for VecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf)?;
        Ok(())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn slice2strvec(s: impl IntoIterator<Item: Into<String>>) -> Vec<String> {
    s.into_iter().map(Into::into).collect()
}

#[derive(Debug)]
struct CargoCallbacksShim {
    rerun_on_header_files: bool,
    // redundant_types: Vec<Regex>,
    pods: Vec<String>,
    trans: Vec<String>,
}
impl CargoCallbacksShim {
    pub fn new() -> Self {
        Self {
            rerun_on_header_files: true,
            pods: slice2strvec(["Pod", "Zeroable"]),
            trans: slice2strvec(["TransparentWrapper"]),
            // redundant_types: REDUNDANT_TYPES
            //     .iter()
            //     .map(|(f, _t)| {
            //         let res = format!("\\b({})+\\b", f);
            //         regex::Regex::new(&res).unwrap()
            //     })
            //     .collect(),
        }
    }
}
impl ParseCallbacks for CargoCallbacksShim {
    fn header_file(&self, filename: &str) {
        if self.rerun_on_header_files {
            println!("cargo:rerun-if-changed={filename}");
        }
    }
    fn include_file(&self, filename: &str) {
        println!("cargo:rerun-if-changed={filename}");
    }

    fn read_env_var(&self, key: &str) {
        println!("cargo:rerun-if-env-changed={key}");
    }
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        let is_trans = TRANSPARENT_WRAPPER.contains(&info.name);
        let is_pod = POD.contains(&info.name);
        if is_pod {
            let mut s = self.pods.clone();
            if is_trans {
                s.push(self.trans[0].clone());
            }
            s
        } else if is_trans {
            self.trans.clone()
        } else {
            Vec::new()
        }
    }
    fn item_name(&self, item_info: bindgen::callbacks::ItemInfo) -> Option<String> {
        match item_info.kind {
            ::bindgen::callbacks::ItemKind::Type => {}
            _ => return None,
        }

        let name = match item_info.name {
            "_bindgen_ty_20" => "QT",
            "_bindgen_ty_21" => "REDIR",
            "_bindgen_ty_22" => "ZCONTEXT",
            "_bindgen_ty_23" => "REDIRF",
            "_bindgen_ty_24" => "VALFLAG",
            "_bindgen_ty_25" => "ASG",
            "_bindgen_ty_26" => "FS",
            "_bindgen_ty_27" => "PARSEARGS",
            "_bindgen_ty_28" => "ZMB",
            "_bindgen_ty_29" => "ZSHTOK",
            "_bindgen_ty_30" => "PREFORK",
            "_bindgen_ty_31" => "ASSPM",
            "_bindgen_ty_33" => "OPT",
            "_bindgen_ty_34" => "TSC",
            "_bindgen_ty_35" => "SORTIT",
            "_bindgen_ty_36" => "CASMOD",
            "_bindgen_ty_37" => "GETKEY",
            "_bindgen_ty_38" => "ZLCON",
            "_bindgen_ty_39" => "ZLE_CMD",
            "_bindgen_ty_40" => "NICEFLAG",

            _ => return None,
        };

        Some(name.to_owned())
    }
    // fn new_item_found(&self, _id: bindgen::callbacks::DiscoveredItemId, item: DiscoveredItem) {
    //     match item {

    //     }
    // }
    // fn generated_name_override(
    //     &self,
    //     item_info: bindgen::callbacks::ItemInfo<'_>,
    // ) -> Option<String> {
    //     match item_info.kind {
    //         ItemKind::Type => {
    //             let underscore = memchr::memchr(b'_', item_info.name.as_bytes())?;
    //             let (pre, _suf) = item_info.name.split_at(underscore);

    //             Some(pre.to_owned())
    //         }
    //         _ => None,
    //     }
    // }
}
const REDUNDANT_TYPES: &[(&str, &str)] = &[
    // ("voidvoidfnptr_t", r#"Option<unsafe extern "C" fn()>"#),
    // (
    //     "VFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_void) -> *mut c_void>"#,
    // ),
    // (
    //     "FreeFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_void)>"#,
    // ),
    // (
    //     "HashFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *const c_char) -> c_uint>"#,
    // ),
    // (
    //     "AddNodeFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut hashtable, arg2: *mut c_char, arg3: *mut c_void)>"#,
    // ),
    // (
    //     "RemoveNodeFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut hashtable, arg2: *const c_char) -> *mut hashnode>"#,
    // ),
    // (
    //     "RemoveNodeFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut hashtable, arg2: *const c_char) -> *mut hashnode>"#,
    // ),
    // (
    //     "FreeNodeFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut hashnode)>"#,
    // ),
    // (
    //     "CompareFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *const c_char, arg2: *const c_char) -> c_int>"#,
    // ),
    // (
    //     "PrintTableStats",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut hashtable)>"#,
    // ),
    // (
    //     "WrapFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: Eprog, arg2: *mut funcwrap, arg3: *mut c_char) -> c_int>"#,
    // ),
    // (
    //     "HandlerFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_char, arg2: *mut *mut c_char, arg3: *mut options, arg4: c_int) -> c_int>"#,
    // ),
    // (
    //     "HandlerFuncAssign",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_char, arg2: *mut *mut c_char, arg3: *mut linkroot, arg4: *mut options, arg5: c_int) -> c_int>"#,
    // ),
    // (
    //     "Module_generic_func",
    //     r#"Option<unsafe extern "C" fn() -> c_int>"#,
    // ),
    // (
    //     "CompareFn",
    //     r#"Option<unsafe extern "C" fn(arg1: *const c_void, arg2: *const c_void) -> c_int>"#,
    // ),
    // (
    //     "CompctlReadFn",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_char, arg2: *mut *mut c_char, arg3: *mut options, arg4: *mut c_char) -> c_int>"#,
    // ),
    // (
    //     "ZleEntryPoint",
    //     r#"Option<unsafe extern "C" fn(cmd: c_int, ap: *mut __va_list_tag) -> *mut c_char>"#,
    // ),
    // (
    //     "NumMathFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_char, arg2: c_int, arg3: *mut mnumber, arg4: c_int) -> mnumber>"#,
    // ),
    // (
    //     "StrMathFunc",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut c_char, arg2: *mut c_char, arg3: c_int) -> mnumber>"#,
    // ),
    // (
    //     "CondHandler",
    //     r#"Option<unsafe extern "C" fn(arg1: *mut *mut c_char, arg2: c_int) -> c_int>"#,
    // ),
    ("Alias", "*mut alias"),
    ("Asgment", "*mut asgment"),
    ("Builtin", "*mut builtin"),
    ("Cmdnam", "*mut cmdnam"),
    ("Complist", "*mut complist"),
    ("Conddef", "*mut conddef"),
    ("Dirsav", "*mut dirsav"),
    ("Emulation_options", "*mut emulation_options"),
    ("Execcmd_params", "*mut execcmd_params"),
    ("Features", "*mut features"),
    ("Feature_enables", "*mut feature_enables"),
    ("Funcstack", "*mut funcstack"),
    ("FuncWrap", "*mut funcwrap"),
    ("HashNode", "*mut hashnode"),
    ("HashTable", "*mut hashtable"),
    ("Heap", "*mut heap"),
    ("Heapstack", "*mut heapstack"),
    ("Histent", "*mut histent"),
    ("Hookdef", "*mut hookdef"),
    ("Imatchdata", "*mut imatchdata"),
    ("Jobfile", "*mut jobfile"),
    ("Job", "*mut job"),
    ("Linkedmod", "*mut linkedmod"),
    ("LinkNode", "*mut linknode"),
    ("LinkList", "*mut linkroot"),
    ("Module", "*mut module"),
    ("Nameddir", "*mut nameddir"),
    ("Options", "*mut options"),
    ("Optname", "*mut optname"),
    ("Param", "*mut param"),
    ("Paramdef", "*mut paramdef"),
    ("Patstralloc", "*mut patstralloc"),
    ("Patprog", "*mut patprog"),
    ("Prepromptfn", "*mut prepromptfn"),
    ("Process", "*mut process"),
    ("Redir", "*mut redir"),
    ("Reswd", "*mut reswd"),
    ("Shfunc", "*mut shfunc"),
    ("Timedfn", "*mut timedfn"),
    ("Value", "*mut value"),
    ("Wordcode", "*mut wordcode"),
    ("FuncDump", "*mut funcdump"),
    ("Eprog", "*mut eprog"),
    ("Estate", "*mut estate"),
    ("Eccstr", "*mut eccstr"),
    ("Zpc_disables_save", "*mut zpc_disables_save"),
    ("GsuScalar", "*const gsu_scalar"),
    ("GsuInteger", "*const gsu_integer"),
    ("GsuFloat", "*const gsu_float"),
    ("GsuArray", "*const gsu_array"),
    ("GsuHash", "*const gsu_hash"),
    ("Repldata", "*mut repldata"),
    ("Groupmap", "*mut groupmap"),
    ("Groupset", "*mut groupset"),
    ("Color_rgb", "*mut color_rgb"),
    ("SortElt", "*mut sortelt"),
    ("__gnuc_va_list", "__builtin_va_list"),
    ("__builtin_va_list", "[__va_list_tag; 1usize]"),
    ("MathFunc", "*mut mathfunc"),
    ("wchar_t", "c_int"),
    ("wint_t", "c_uint"),
    ("wctype_t", "c_ulong"),
    ("__priority_which", "c_uint"),
];
/*


*/

const REPLACEMENTS: &[(&str, &str)] = &[
    (" ::", " "),
    ("(::", "("),
    ("[::", "["),
    ("{::", "{"),
    ("&::", "&"),
    ("*::", "*"),
    ("core::mem::", ""),
    ("core::ffi::", ""),
    ("core::ops::", ""),
    ("core::fmt::", ""),
    ("core::ptr::addr_of!", "&raw const"),
    ("core::ptr::addr_of_mut!", "&raw mut"),
    ("core::option::Option", "Option"),
    ("core::marker::PhantomData", "PhantomData"),
    ("pub type _bindgen_ty", "pub(crate) type _bindgen_ty"),
    // ("pub use self::_", "pub(crate) use self::_"),
];

const POD: &[&str] = &[
    "color_rgb",
    "timeinfo",
    "multio",
    "entersubsh_ret",
    "noerrexit_bits",
];
const TRANSPARENT_WRAPPER: &[&str] = &["noerrexit_bits"];

fn get_outdir() -> PathBuf {
    let mut out_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // println!("cargo::warning=Using out_path: {:?}", out_path);

    out_path.push("src");
    out_path.push("bindings.rs");
    out_path
}

fn create_bindings() -> String {
    let mut builder = bindgen::Builder::default()
        .header("headers/wrapper.h")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .prepend_enum_name(false)
        .default_macro_constant_type(::bindgen::MacroTypeVariation::Signed)
        .fit_macro_constants(false)
        .generate_comments(true)
        .parse_callbacks(Box::new(CargoCallbacksShim::new()))
        .use_core()
        .generate_cstr(true)
        .merge_extern_blocks(true)
        .wrap_unsafe_ops(true)
        .blocklist_file("/usr/include/(sys/|).*.h")
        .layout_tests(false)
        .anon_fields_prefix("anon")
        .raw_line(
            "#![allow(nonstandard_style, dead_code, improper_ctypes, unnecessary_transmutes)]
use {bytemuck::*, core::{ops::*, ffi::*, mem::*, fmt::*}, libc::*};",
        )
        .derive_copy(true)
        .impl_debug(true)
        .default_non_copy_union_style(::bindgen::NonCopyUnionStyle::ManuallyDrop)
        .ctypes_prefix("")
        .opaque_type("alias")
        .blocklist_file(
            r"headers/(zshcurses|zshpaths|zsh_system|patchlevel|prototypes|zshterm|zshxmods).h",
        )
        .blocklist_file("../config.h")
        .blocklist_item(r"(HAVE|GLOBAL|ZSH_HAVE|USE|PACKAGE|true|false)_\S*")
        .blocklist_item(
            r"(PASSWD_FILE|_XOPEN_SOURCE_EXTENDED|_GNU_SOURCE|__bool_true_false_are_defined)",
        )
        .blocklist_type("max_align_t")
        .bitfield_enum("noerrexit_bits")
        .sort_semantically(true)
        .formatter(::bindgen::Formatter::Rustfmt);
    // .allowlist_file("/usr/include/wctype.h")
    // .ctypes_prefix("")
    for (f, _r) in REDUNDANT_TYPES {
        builder = builder.blocklist_type(f.trim_ascii());
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let mut filestuff = VecWriter(Vec::with_capacity(8192));
    bindings.write(Box::new(&mut filestuff)).unwrap();

    let as_string = String::from_utf8(filestuff.0).unwrap();
    as_string
}

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=headers/wrapper.h");
    let out_path = get_outdir();

    let filestuff = create_bindings();

    let filestuff = REPLACEMENTS
        .iter()
        .fold(filestuff, |acc, (f, t)| acc.replace(f, t));

    let filestuff = REDUNDANT_TYPES
        .iter()
        .map(|(f, t)| {
            let res = format!("\\b({})+\\b", f);
            (regex::Regex::new(&res).unwrap(), t)
        })
        .fold(filestuff, |acc, (re, t)| {
            re.replace_all(&acc, *t).into_owned()
        });

    // let filestuff = regex::RegexBuilder::new(
    //     r"^(\s*pub\s*const\s*\S+\s*:\s*_bindgen_ty_[0-9]+\s*=\s*_bindgen_ty_[0-9]+\S+\s*;\s*)$",
    // )
    // .multi_line(true)
    // .build()
    // .unwrap()
    // .replace_all(&filestuff, "")
    // .lines()
    // .filter(|l| !l.trim().is_empty())
    // .collect::<Vec<_>>()
    // .join("\n");

    std::fs::write(out_path, filestuff)?;

    Ok(())
}
