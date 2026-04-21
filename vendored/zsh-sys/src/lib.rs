//! # Zsh Rust Bindings
//!
//! This crate provides Rust bindings to the Z shell (zsh) C API.
//! It exposes the core data structures, functions, and constants used
//! by zsh internals, enabling Rust code to interface with zsh's core functionality.
//!
//! ## Overview
//!
//! The zsh shell is implemented in C, and this crate provides a safe Rust interface
//! to its core components. The bindings include:
//!
//! - Core data structures for shell internals
//! - Function interfaces for shell operations
//! - Constants for shell options, tokens, and flags
//! - Memory management utilities
//! - Job control and process management
//! - Parameter and variable handling
//! - History and completion systems
//! - Terminal and display control
//! - Module loading and hook systems
//!
//! ## Key Components
//!
//! ### Data Structures
//!
//! The module defines Rust equivalents of zsh's internal data structures:
//!
//! - **Parameter Management**: `Param`, `Paramdef`, `Value`, `HashTable`
//! - **Job Control**: `Job`, `Process`, `jobfile`
//! - **Parsing**: `Eprog`, `Estate`, `Wordcode`, `linknode`
//! - **History**: `Histent`, `histent`
//! - **Modules**: `Module`, `Features`, `Hookdef`
//! - **Patterns**: `Patprog`, `Patstralloc`
//! - **Builtins**: `Builtin`, `builtin`
//! - **Functions**: `Shfunc`, `funcstack`, `FuncWrap`
//!
//! ### Constants and Enums
//!
//! Many constants from zsh's header files are exposed:
//!
//! - **Shell Options**: `ALIASESOPT`, `AUTOCD`, etc.
//! - **Lexical Tokens**: `lextok`, `NULLTOK`, `SEMI`, etc.
//! - **Redirection Types**: `REDIR_WRITE`, `REDIR_READ`, etc.
//! - **Job Status**: `STAT_CHANGED`, `STAT_STOPPED`, etc.
//! - **Pattern Flags**: `PAT_FILE`, `PAT_NOANCH`, etc.
//! - **Math Functions**: `MFF_STR`, `MFF_ADDED`, etc.
//!
//! ### Memory Management
//!
//! The module provides Rust wrappers for zsh's memory management:
//!
//! - `new_heaps()`, `old_heaps()`, `switch_heaps()` for heap switching
//! - `zhalloc()`, `zrealloc()`, `zfree()` for allocation
//! - `pushheap()`, `popheap()` for stack-based memory management
//!
//! ### Core Functionality
//!
//! Key functions for shell operations:
//!
//! - **Execution**: `execstring()`, `execode()`, `execlist()`
//! - **Parsing**: `parse_string()`, `parse_list()`, `parse_cond()`
//! - **Builtins**: `execbuiltin()`, `bin_set()`, `bin_unset()`
//! - **Jobs**: `addproc()`, `waitjobs()`, `killjb()`
//! - **History**: `hgetline()`, `addhistnum()`, `gethistent()`
//! - **Parameters**: `assignstrvalue()`, `getnparam()`, `setaparam()`
//! - **Modules**: `load_module()`, `register_module()`, `addhookdef()`
//!
//! ## Usage
//!
//! This crate is intended for low-level integration with zsh internals.
//! It's typically used when creating zsh extensions, plugins, or when
//! implementing shell-like functionality in Rust.
//!
//! ## Safety
//!
//! The bindings are marked as `unsafe` where appropriate, especially
//! when:
//!
//! - Accessing C memory directly
//! - Calling C functions that may modify global state
//! - Handling raw pointers and memory management
//!
//! ## Limitations
//!
//! - This is a low-level binding, not a high-level shell interface
//! - Some zsh features may not be fully exposed
//! - Memory management requires careful handling
//! - The API may change between zsh versions
//!
//! ## See Also
//!
//! - [Zsh Manual](https://zsh.sourceforge.org/Doc/Release/)
//! - [Zsh Source Code](https://sourceforge.net/projects/zsh/)
//! - [Rust Bindgen](https://github.com/rust-lang/rust-bindgen)
//!
//! # Generated Bindings
//!
//! This module was automatically generated using rust-bindgen from zsh's C headers.
//! The following sections describe the generated bindings and their usage.
//!
//! ## Generated Types
//!
//! The module defines a large number of generated types that correspond to zsh's
//! C structures. These include:
//!
//! - `mnumber`, `mathfunc`, `entersubsh_ret`, `complist`
//! - `linknode`, `linklist`, `prepromptfn`, `timedfn`
//! - `conddef`, `redir`, `multio`, `value`
//! - `funcdump`, `eprog`, `estate`, `eccstr`
//! - `jobfile`, `job`, `process`, `execstack`, `heredocs`
//! - `hashtable`, `hashnode`, `optname`, `reswd`, `alias`
//! - `asgment`, `cmdnam`, `shfunc`, `funcstack`, `funcwrap`
//! - `options`, `builtin`, `execcmd_params`, `module`, `linkedmod`
//! - `features`, `feature_enables`, `hookdef`, `patprog`, `patstralloc`
//! - `zpc_disables_save`, `gsu_scalar`, `gsu_integer`, `gsu_float`, `gsu_array`, `gsu_hash`
//! - `param`, `tieddata`, `repldata`, `paramdef`, `nameddir`, `groupmap`, `groupset`
//! - `histent`, `emulation_options`, `ttyinfo`, `color_rgb`, `heapstack`, `heap`, `sortelt`
//! - `hist_stack`, `lexbufstate`, `lex_stack`, `parse_stack`
//!
//! ## Generated Constants
//!
//! A comprehensive set of constants representing zsh's configuration and behavior:
//!
//! - `PASSWD_FILE`, `GLOBAL_ZLOGIN`, etc. - File paths and system constants
//! - `DEFAULT_HISTSIZE`, `DEFAULT_FCEDIT`, etc. - Default configuration values
//! - `ALIASESOPT`, `AUTOCD`, etc. - Shell option flags
//! - `lextok` enum values - Token types for parsing
//! - `REDIR_*` constants - Redirection types
//! - `STAT_*` flags - Job status flags
//! - `PAT_*` flags - Pattern matching flags
//! - `MFF_*` flags - Math function flags
//! - `QT_*` enum values - Quote types
//! - And many more...
//!
//! ## Generated Functions
//!
//! The module exposes numerous functions for interacting with zsh internals:
//!
//! - **Memory Management**: `zhalloc`, `zrealloc`, `zfree`, `new_heaps`
//! - **Execution**: `execstring`, `execode`, `execlist`, `parse_string`
//! - **Builtins**: `execbuiltin`, `bin_set`, `bin_unset`, `bin_cd`
//! - **Jobs**: `addproc`, `waitjobs`, `killjb`, `printjob`
//! - **History**: `hgetline`, `addhistnum`, `gethistent`, `savehistfile`
//! - **Parameters**: `assignstrvalue`, `getnparam`, `setaparam`, `unsetparam`
//! - **Modules**: `load_module`, `register_module`, `addhookdef`
//! - **Parsing**: `zshlex`, `parse_list`, `parse_cond`, `patcompile`
//! - **Terminal**: `init_term`, `putstr`, `tsetcap`, `adjustwinsize`
//! - **I/O**: `zputs`, `nicezputs`, `readoutput`, `getoutput`
//! - And many more...
//!
//! ## Generated Variables
//!
//! Global variables used throughout zsh:
//!
//! - `opts`, `emulation`, `locallevel` - Shell options and state
//! - `jobtab`, `curjob`, `maxjob` - Job control variables
//! - `paramtab`, `shfunctab`, `builtintab` - Hash tables
//! - `prompt`, `prompt2`, `prompt3`, `prompt4` - Prompt strings
//! - `home`, `pwd`, `path` - Environment variables
//! - `sigtrapped`, `siglists` - Signal handling
//! - And many more...
#![no_std]

mod bindings;
pub use bindings::*;

pub mod macros;
