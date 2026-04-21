# Zsh Internal Systems Manual
## Developer Reference: Implementation-Level Architecture

---

## 1. CORE ARCHITECTURE

### Runtime Model
- **Single-threaded, cooperative task execution** with selective forking
- **Event loop driven** via `loop()` in `init.c`; processes commands sequentially
- **Subshells use `vfork()` + `exec()`** pattern via `entersubsh()` → `addproc()`
- **Command execution pipeline**: lex → parse → compile → execute

### Global State Layout (`zsh.h` + `init.c`)
- **`extern int noerrexit`** (`exec.c`): suppresses `ERREXIT` flag
- **`extern int this_noerrexit`** (`exec.c`): localized suppression
- **`extern sigset_t sigchld_mask`** (`zsh.h`): SIGCHLD blocking mask
- **`extern struct hookdef zshhooks[]`** (`init.c`): hook dispatch table
- **`extern int sourcelevel`** (`init.c`): depth of sourced files
- **`extern int inwhat`** (`lex.c`): lexer context (IN_*)  
- **`extern int errflag`** (`init.c`): cumulative error flags

### Major Subsystems
| Subsystem | Primary Source Files | Key Entry Points |
|-----------|---------------------|------------------|
| **Parser** | `parse.c`, `lex.c` | `parse()` (via `zshlex()`), `parse_command()` |
| **Execution** | `exec.c`, `exec.h` | `execlist()`, `execcmd()` |
| **ZLE** | `Zle/` directory | `zle_main()`, `zle_get_buffer()` |
| **Completion** | `Zle/compcore.c`, `Zle/complete.c` | `_main_complete()`, `compgen()` |
| **Modules** | `module.c` | `zmodload`, `setup_module()`, `boot_module()` |
| **Signals** | `signals.c`, `signals.h` | `zhandler()`, `queue_signals()` |

### Control Flow
```
main() → zsh_main() → init() → loop() 
  → zshlex() → parse() → compile() → execlist() → execcmd()
       ↑                         |
       └─ zle_dispatch() ←------┘
       └─ _main_complete() ←────┘
```

### State Mutation Hotspots
- **`tokstr`** (lex.c): mutable token buffer, reallocates on growth
- **`lexbuf`** (lex.c): grows dynamically via `hrealloc()` on buffer overflow
- **`noerrexit`/`this_noerrexit`** (exec.c): set/cleared around sub-command evaluation
- **`inwhat`** (lex.c): context-dependent, mutated per lexical scope
- **Module state flags** (`MOD_SETUP`, `MOD_INIT_B`, etc.): bitmask in `struct module`

---

## 2. MEMORY MODEL

### Allocation Systems
- **`zalloc(size)`**: internal wrapper over `malloc()`; use for all runtime allocations
- **`zrealloc(ptr, old, new)`**: wrapper over `realloc()`; returns new pointer
- **`zfree(ptr, size)`**: wrapper over `free()`; size used for debugging
- **`pushheap()` / `popheap()` / `freeheap()`**: scoped memory arenas
  - `pushheap()`: marks current heap top
  - `popheap()`: frees all allocations since last `pushheap()`
  - `freeheap()`: frees entire heap (used sparingly)

### Heap Management Patterns
```c
pushheap();
Eprog prog = (Eprog)zalloc(sizeof(*prog));
prog->prog = (Wordcode)zalloc(p->len);
popheap();  // prog and prog->prog now freed together
```

### Ownership Rules
- **No formal ownership semantics**: convention-based only
- **`LBUFFER`/`RBUFFER`**: owned by lexer (`lexbuf`); mutated in-place, reallocates on growth
- **Command argument arrays**: owned by executor; freed via `freeprog()` → `freeeprog()`
- **Module-loaded memory**: caller must explicitly free; module `cleanup_` may free

### Lifetimes & Invalidation
| Buffer | Owner | Invalidation Trigger |
|--------|-------|---------------------|
| `tokstr` | Lexer | Next call to `gettok()` / `zshlex()` |
| `lexbuf.ptr` | Lexer | Buffer reallocation (growth) |
| `prog->prog` | Executor | `freeprog()` called |
| `prog->strs` | Executor | `freeprog()` called |
| `value.arr` | Caller | Next value assignment or `freeprog()` |
| `zunderscore` (`$_`) | Shell | Next assignment |

### Key Patterns to Prevent UB
- **Never hold pointers across `pushheap()`/`popheap()` boundaries** unless the arena is guaranteed persistent
- **Always use `zalloc`/`zfree`** matching pairs (every `zalloc` must have exactly one `zfree`)
- **After `freeprog()`, all embedded pointers (`prog->prog`, `prog->strs`, `prog->pats`) are invalid**
- **`tokstr` is valid until the next call to `gettok()` or `zshlex()`**

---

## 3. MODULE SYSTEM

### `struct module`
```c
struct module {
    struct hashnode node;
    union {
        void *handle;        // dlopen handle (dynamic)
        Linkedmod linked;    // linked module descriptor
        char *alias;         // alias name
    } u;
    LinkList autoloads;      // autoload feature names
    LinkList deps;           // dependent module names
    int wrapper;             // module wrapper flag
};
```

### Lifecycle Functions
| Function | When Called | Guarantees | Forbidden Actions |
|----------|-------------|------------|-------------------|
| **`setup_`** | Module registration via `zmodload` | `zmodload` has parsed module name; module record exists; heap is stable | No signal handling; no ZLE buffer access; no command execution |
| **`boot_`** | First actual use (autoload or feature enable) | Global state initialized; module handles loaded; heap intact | Should not depend on `setup_` having been called (may be skipped) |
| **`cleanup_`** | Module removal (`zmodload -R`) | Module still functional; references still valid | Do NOT free `u.handle` or `u.linked` (caller does) |
| **`finish_`** | Shell exit (`FINISH` flag) | Shell is shutting down; no further commands | Must NOT call any shell functions; only cleanup |

### State Validity Matrix
| Phase | `setup_` | `boot_` | `cleanup_` | `finish_` |
|-------|----------|---------|------------|-----------|
| `zmodload -a` | ✓ | | | |
| `zmodload` (autoload) | ✓ | ✓ | | |
| `zmodload -R` | | | ✓ | |
| Shell exit | | | | ✓ |

### Reentrancy & Reload Behavior
- **Modules may be re-`boot_`'d after `cleanup_`** if re-enabled
- **`setup_` is called at most once per module record**; subsequent loads skip to `boot_`
- **Reloading (setup → boot → cleanup → boot) is supported** but `u.handle` must remain valid or be reopened
- **Signal handlers MUST NOT call module functions** (not async-signal-safe)

---

## 4. ZLE (LINE EDITOR) INTERNALS

### Buffer Management
- **Primary buffers**: `lexbuf` (for lexing), `tokstr` (for current token)
- **LBUFFER/RBUFFER**: Internal names for input/output buffers in ZLE widget implementations
  - Accessed via `zle_get_buffer()` / `zle_set_buffer()` (not directly exported)
  - Reallocates via `hrealloc()` when content exceeds capacity
- **Widget dispatch**: `zle_main()` → widget function → modifies `lexbuf` or `tokstr`

### Widget Dispatch Model
```
zle_main()
  → get_token() → tok = lex()
    → dispatch_widget(tok, args)
      → widget_func()  // may call zle_set_buffer(), zle_reset_prompt()
```

### Hook System
| Hook Name | Trigger Point | Safe Actions |
|-----------|---------------|--------------|
| `line-init` | Before first widget | Read-only buffer inspection |
| `line-pre-redraw` | Before redraw | Modify buffer, set `REDRAW_ACTIVE` |
| `line-finished` | After command execution | Cleanup, status updates |

### Critical Invariants
- **Buffer pointers (`lexbuf.ptr`, `tokstr`) invalid after buffer reallocation**
- **ZLE is single-threaded per input stream** but may be re-entered via subshells
- **Signal handlers (`TRAP*`)** run asynchronously; they MUST NOT modify ZLE buffers directly
- **Widget functions must not call `zshlex()` recursively** (would corrupt `inwhat` state)

---

## 5. COMPLETION SYSTEM

### Trigger Path
```
user presses TAB
  → zle_main() → do_complete()
    → _main_complete()
      → compgen() / compctl()
        → matches → compadd() → compstate
```

### Key Variables
- **`_main_complete`**: Top-level completion dispatcher (`Zle/_main_complete`)
- **`compadd`**: Builtin builtin; adds matches to completion list
- **`compstate`**: Associative array with keys:
  - `state`, `state_desc`, `menu`, `current`, `list`, `listmax`, `match`, `matches`
  - `insert`, `unambiguous`, `menu_complete`, `remove`

### Data Flow Semantics
- **`compadd` returns 0** on success, 1 if no matches
- **Matches accumulate** in global `preexp` / `match_list` arrays
- **`do_complete()` sets `dolastprompt`** to return to prompt on failure
- **Side effects**: Modifies `compstate`, may call widgets (menucompletion)

### Hidden Contracts
- **`compadd` expects `preexp` to be set** before it is called
- **`compstate[insert]` is a group number** used by `_main_complete` to select match
- **Completion functions MUST NOT call `execlist()`** (would corrupt completion state)
- **`oldlist` flag** prevents discarding previous completions across widget calls

---

## 6. GLOBAL STATE AND SIDE EFFECTS

### Major Global Variables
| Variable | File | Volatility | Concurrency Risk |
|----------|------|------------|------------------|
| `noerrexit` | `exec.c` | High (set/cleared per command) | Sub-command nesting |
| `this_noerrexit` | `exec.c` | Very High (per-expression) | Deeply nested |
| `inwhat` | `lex.c` | Per-lexeme | Reentrant via subshells |
| `tokstr` | `lex.c` | High (per-token) | Reentrant |
| `lexbuf` | `lex.c` | High (grows) | Reentrant |
| `zcontext` | `init.c` | Per-scope | Signal handlers |
| `sigchld_mask` | `zsh.h` | Stable | Signal handlers |

### State Leakage Patterns
- **`inwhat` leaks across sourced files**: Modifications in sourced script persist
- **Completion state (`compstate`) is global**: Not reset between completions
- **`noerrexit` persists until explicitly cleared**: Can block error reporting
- **Module flags persist across `cleanup_` → `boot_` transitions**

### Invariant Assumptions (Sometimes Broken)
- **"Command arguments are stable during execution"**: False when command forks
- **"ZLE buffer is stable during widget execution"**: False if widget spawns subshell
- **"Module setup is called before any boot code"**: False if module autoloads mid-execution

---

## 7. SIGNALS AND ASYNC BEHAVIOR

### Signal Handling
- **`TRAP*` signals** implemented via `zhandler()` in `signals.c`
- **Signal context**: Runs asynchronously; must be async-signal-safe
- **Queued signals**: Use circular queue (`signal_queue[]`, `queue_front/rear`)

### ASYNC-SIGNAL-SAFE Functions Only
- `write()`, `signal()`, simple flag checks
- **NOT SAFE**: `zalloc()`, `zfree()`, `zshlex()`, `printf()`, heap ops

### Critical Unsafe Patterns
```c
// UNSAFE: signal handler calling ZLE
void trap_handler() {
    zle_main(...);  // ← BUG: not async-signal-safe
}

// UNSAFE: signal handler modifying completion state
void intr_handler() {
    compstate["current"] = 0;  // ← BUG: not async-signal-safe
}

// SAFE: signal handler just sets flag
volatile sig_atomic_t got_int = 0;
void intr_handler() {
    got_int = 1;  // ← OK: atomic on most platforms
}
```

### Interaction with ZLE and Completion
- **Signals during ZLE**: May interrupt `zshlex()` → partial token in `lexbuf`
- **Signals during completion**: Can leave `compstate` in inconsistent state
- **`queue_signals()` / `unqueue_signals()`**: Used to defer handling across critical sections

---

## 8. THREAD SAFETY (OR LACK THEREOF)

### Explicit Answer: Zsh is NOT thread-safe
- **No internal mutexes or synchronization primitives**
- **All global state is shared mutable**
- **Modules that spawn threads will corrupt state**

### What Breaks in Multithreaded Contexts
- **`lexbuf` reallocation** while another thread is reading token
- **Completion list mutation** while another thread iterates matches
- **Module `boot_` / `cleanup_`** concurrent with command execution
- **`noerrexit` modification** between command evaluation and error check

### What Happens If Modules Spawn Threads
- **Undefined behavior**: data races on all global state
- **Likely crash**: heap corruption from concurrent `zalloc`/`zfree`
- **Possible deadlock**: signal handlers interacting with thread locks
- **No recovery**: entire process state is unsound

### Safe Multithreading Pattern (If You Must)
```c
// Use process isolation, NOT threads
pid_t pid = fork();
if (pid == 0) {
    // child: safe to call Zsh functions
    run_command_in_child();
    _exit(0);
} else {
    // parent: wait for child, no shared state
    waitpid(pid, ...);
}
```

---

## 9. COMMON EXTENSION PITFALLS

| Pitfall | Cause | Example Scenario | Mitigation |
|---------|-------|------------------|------------|
| **Buffer invalidation** | Reallocating `lexbuf`/`tokstr` | Holding pointer across `gettok()` | Copy data immediately or use indices |
| **Use-after-free** | Assuming `prog` stability after `freeprog()` | Storing `prog->prog` for later | Never cache embedded pointers |
| **Reentrancy bug** | Calling `execlist()` from widget | Widget spawned via `eval` | Use flags to prevent recursion |
| **FD leak** | Opening pipes/files in module | `open()` without `close()` in `cleanup_` | Always pair opens with closes |
| **Signal corruption** | Modifying state in `TRAP` handler | Changing `compstate` in trap | Defer with queued signals |
| **Module reload race** | `boot_` called while in use | Autoload during command execution | Check `MOD_INIT_B` before use |
| **Heap mismatch** | Mixing `zalloc`/`malloc` | `zalloc` then `free` (not `zfree`) | Use consistent allocators |
| **Invariant violation** | Assuming single-threaded | Spawning threads in module | Use fork, not threads |

### Detailed Scenarios
1. **Widget Buffer Corruption**
   - **Cause**: Widget calls `zle_set_buffer()` which reallocates `lexbuf`
   - **Effect**: Any cached `lexbuf.ptr` becomes dangling
   - **Fix**: Use `zle_get_buffer()` → copy → modify → `zle_set_buffer()`

2. **Completion State Leak**
   - **Cause**: Completion function modifies global `preexp` permanently
   - **Effect**: Next completion starts with dirty state
   - **Fix**: Save/restore `compstate` around custom completion code

3. **Module Unload Crash**
   - **Cause**: `cleanup_` frees memory still referenced by active command
   - **Effect**: Segfault when command tries to use freed memory
   - **Fix**: Use reference counting or ensure command finishes before unload

---

## 10. FFI BOUNDARY GUIDELINES (FOR RUST/C++)

### Rules for Safely Calling Zsh from Foreign Code
- **Treat all Zsh pointers as `unsafe`**: `Module`, `Eprog`, `Wordcode`, `char*`
- **Establish clear ownership**: Who allocates, who frees?
- **Copy out pointers on FFI boundary**: Never pass Zsh pointers across FFI
- **Pin Rust references** if you must store Zsh pointers: `Pin<&'static mut T>`

### What Must Remain `unsafe`
```rust
// UNSAFE: All Zsh C API calls are inherently unsafe
extern "C" {
    fn zshlex() -> *const c_char;  // Returns pointer into Zsh heap
    fn zalloc(size: usize) -> *mut c_void;
    fn freeprog(p: *mut c_void);
}
```

### Recommended Abstraction Patterns
#### Pattern 1: Copy-Out for Strings
```rust
// Safe wrapper
fn get_zsh_string(zstr: *const libc::c_char) -> String {
    unsafe { CStr::from_ptr(zstr).to_string_lossy().into_owned() }
}
```

#### Pattern 2: Arena-Based Lifetime
```rust
struct ZshArena<'a> {
    _zsh: PhantomData<&'a ()>,  // Ensures Zsh is alive
    ptr: *mut libc::c_void,
}
impl<'a> Drop for ZshArena<'a> {
    fn drop(&mut self) {
        unsafe { zfree(self.ptr, /*size*/) };
    }
}
```

#### Pattern 3: State Machine for Lifetimes
```rust
enum ZshState<'a> {
    Initialized(*mut ZshRuntime),
    Running(&'a mut ZshRuntime),
    Finished,
}
// Transition enforces lifetime rules at type level
```

### Lifetime Management
- **Never store Zsh pointers across FFI calls** unless you control the entire call chain
- **Use indices, not pointers, for cross-FFI references** (e.g., `usize` offsets into a known arena)
- **When borrowing**: Ensure Zsh code cannot outlive the borrowed data
- **When owning**: Use `Box`/`Vec` in Rust, transfer ownership to Zsh with explicit `zfree` contract

---

## 11. SUBSYSTEM-SPECIFIC CONTRACTS

### Parser Subsystem (`parse.c`, `lex.c`)
**Invariants Expected**
- `tokstr` points to valid null-terminated string
- `lexbuf` has at least 1 byte capacity
- `inwhat` is one of `IN_CMD`, `IN_EXPR`, `IN_SUBST`, etc.

**Invariants Broken By**
- Signal handlers modifying `lexbuf.ptr`
- External code calling `zshlex()` directly
- Buffer reallocation without updating all references

**External Code Must Respect**
- Do not call `zshlex()` directly; use `gettok()` wrapper
- Do not modify `tokstr` or `lexbuf` directly
- Assume `tok` and `zshlextext` are valid until next `gettok()`

### Execution Subsystem (`exec.c`)
**Invariants Expected**
- `noerrexit` and `this_noerrexit` are properly nested
- `pipestats` array matches current pipeline depth
- `exestack` is properly balanced (push/pop pairs)

**Invariants Broken By**
- Forked children modifying parent's `noerrexit`
- Asynchronous signal delivery during `execlist()`
- Subshell `entersubsh()` creating unpaired execstack entries

**External Code Must Respect**
- Never modify `noerrexit` directly; use `noerrexit_bits`
- Assume pipe FDs in `pipestats` remain valid until pop
- Signal handlers must not call `execlist()`-related functions

### Completion Subsystem (`Zle/compcore.c`, `Zle/complete.c`)
**Invariants Expected**
- `compstate["state"]` is one of `menu`, `list`, `select`, etc.
- `preexp` matches current completion context
- Matches list is either empty or contains valid strings

**Invariants Broken By**
- Widget functions that don't follow `compadd` protocol
- Direct modification of `compstate` without `_main_complete` coordination
- Completion functions calling arbitrary shell code

**External Code Must Respect**
- Always pair `compadd` calls with proper `compstate` setup
- Never call `_main_complete` from signal handler
- Assume completion state is invalid after any shell function call

### Module Subsystem (`module.c`)
**Invariants Expected**
- Module `node.flags` reflects current lifecycle stage
- `MOD_BUSY` prevents concurrent `setup_`/`boot_`
- `linkedmodules` list is consistent

**Invariants Broken By**
- Race conditions in `dyn_` functions
- Calling module functions with wrong state
- Circular dependency in `deps` list

**External Code Must Respect**
- Check `MOD_SETUP`/`MOD_INIT_B` before calling module functions
- Never call `setup_` directly; use `setup_module()`
- Respect `MODULE_PATH` when loading external modules