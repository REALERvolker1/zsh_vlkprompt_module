# Question

Parameters seem to require `*mut c_char` string names and `*mut c_void`-boxed data.
Should we use `zhalloc`? Should we do it with `zalloc`?

Essentially, if I declare variables in a script like so:

```zsh
typeset -a myarr=("scalar1" scalar2)

typeset -i myint=0

mparam=''

if (($+SOME_ENV_VAR)); then
    typeset -i mparam
fi

myfn() {
    local -A mytable=(
        [k]='hello'
        [v]='world'
    )
    print -- "$@"
    printf '%s: %s\n' ${(kv)mytable}
    # does `mytable` get "dropped" here? What is going on?
}

myfn hii

# does `myfn` get freed?
unfunction myfn

# Which allocator does `-g` use? Does it use `zalloc` or a heap?
typeset -gE HAS_COLOR=1.2

# is `i` a scalar, a raw `zlong`, or a `mnumber`?
for ((i = 0; i < 5; ++i)); do
    print -p "%B${i}%b"
done

# This uses a custom GSU! I know this!
echo $RANDOM

# Seed RANDOM with a dynamic variable!
RANDOM=$((${#commands[@]} + $SECONDS))

# I'm not even going to try to describe "tied" variables

```

What on earth is going on behind the scenes? Go line by line.

# zsh parameters: allocation, ownership, scope, and what the sample script does

This note is source-derived from `gitignored/zsh/Src/params.c`, `builtin.c`,
`exec.c`, `math.c`, `hashtable.c`, `utils.c`, `mem.c`, and `zsh.h`. It is
written for this Rust module: the practical question is whether names and values
passed through our FFI should come from `zhalloc()` or `zalloc()`.

## Short answer

### Parameter names

Use depends on which zsh API you are calling.

- For ordinary calls such as `createparam(name, flags)`, `setsparam(name, val)`,
  `setiparam(name, val)`, `setaparam(name, val)`, and `sethparam(name, pairs)`,
  the **parameter name is only an input key**. `createparam()` installs the
  actual table key by doing `paramtab->addnode(paramtab, ztrdup(name), pm)`.
  So the installed `pm->node.nam` is permanent zsh allocation regardless of how
  the input `name` was allocated.

- For plain, unsubscripted names, the input name may be a static C string or a
  stack buffer. It does **not** need `zhalloc()` or `zalloc()`.

- For names containing subscripts (`foo[1]`, `hash[$key]`), assignment helpers
  temporarily write into the name string while parsing (`*ss = '\0'`, later
  restored). Those inputs must be mutable. Do not pass a read-only C literal
  as a subscripted assignment string.

- For `Paramdef` / module feature tables, `d->name` is retained in the module's
  `paramdef` and later reused by `deleteparamdef()`, `featuresarray()`, feature
  enable/disable logic, etc. It must live for the lifetime of the module
  feature definition. Use a static C string. If it cannot be static, use
  permanent storage (`ztrdup` / `zalloc`), not `zhalloc`.

### Parameter values / boxed data

For data that becomes the value of a real parameter, use zsh **permanent**
storage (`zalloc`, `zshcalloc`, `ztrdup`, `zarrdup`, `mkarray`,
`newparamtable`) or static storage for special parameters that point at your own
variables.

Do **not** use `zhalloc()` for any string, array, hash table, or boxed backing
object that the parameter will own after the current heap frame. zsh heap memory
is temporary; it is discarded by `popheap()` / `OLDHEAPS` and is used for parse,
expansion, scan, and formatting temporaries. Ordinary parameter values are later
freed with `zsfree()`, `freearray()`, `deleteparamtable()`, or GSU-specific
unsets, so handing zsh a heap pointer would become use-after-free or an invalid
free.

### For this project's constraints

The zsh source calls `zalloc()`, `ztrdup()`, `zshcalloc()`, etc. for parameter
storage, but those are zsh's permanent allocator (ultimately malloc-like). Our
project prefers stack-only memory. That means:

- Passing static names such as `c"EXCOUNT".as_ptr().cast_mut()` is fine for
  module `Paramdef` names.
- Static Rust variables used as `Paramdef.var` backing data are fine.
- Creating ordinary dynamic zsh parameters with string/array/hash values cannot
  be done correctly without giving zsh permanent allocation to own.

## The model in zsh

A zsh parameter is a `struct param` stored in `paramtab`, a hash table whose node
key is `pm->node.nam`.

`struct param` has:

- hash node header: name, flags, next pointer;
- `u`: a union whose active member depends on `PM_TYPE(flags)`:
    - `u.str` for scalar / nameref strings;
    - `u.arr` for arrays;
    - `u.hash` for associative arrays;
    - `u.val` for integers;
    - `u.dval` for floats;
    - `u.data`, `u.valptr`, etc. for special parameters whose GSU points
      elsewhere;
- `gsu`: get/set/unset table selected by type;
- metadata: base, width, env string, alternate tied name, hidden old parameter,
  and local scope level.

`PM_SCALAR` is `0`; type is selected by `PM_TYPE(flags)` and not by looking at
which union member happens to be non-null. Integers are raw `zlong` in the
parameter node for normal integer parameters, but arithmetic evaluation uses
`mnumber`, a tagged union (`MN_INTEGER`, `MN_FLOAT`, `MN_UNSET`) while evaluating
expressions.

## Permanent allocation vs zsh heap allocation

zsh has two allocation families that matter here:

- Permanent storage: `zalloc()`, `zshcalloc()`, `ztrdup()`, `zarrdup()`,
  `mkarray()`, `newparamtable()`.
    - This is for objects with explicit lifetime.
    - Parameter nodes and parameter-owned values live here.
    - Freed with `zfree()`, `zsfree()`, `freearray()`, `deleteparamtable()`, or a
      specialized GSU unset/set function.

- Heap storage: `zhalloc()`, `hcalloc()`, `dupstring()`, `hmkarray()`, and many
  expansion helpers.
    - This is scoped scratch memory.
    - Freed wholesale by heap pops; callers generally do not individually free it.
    - Used for temporary `Value` objects, scan results, expansion arrays, parsed
      strings, function call frame scratch, and formatting output.

The key invariant in `params.c`: anything copied into a real parameter is made
permanent before installation. Examples:

- `createparam()` allocates a new normal `Param` with `zshcalloc(sizeof *pm)` and
  installs `ztrdup(name)` into the table.
- `copyparam()` says values copied into a real parameter “must be permanently
  allocated,” then uses `ztrdup`, `zarrdup`, or `copyparamtable`.
- `strsetfn()` frees the old `pm->u.str` with `zsfree()` and stores the incoming
  pointer as the new owner-owned value.
- `arrsetfn()` frees the old array with `freearray()` and stores the incoming
  `char **`.
- `hashsetfn()` deletes the old parameter table and stores the incoming one.

Temporary APIs return heap pointers. For example, `getstrvalue()` and hash scan
helpers can return `zhalloc()` results. Do not cache those beyond the current
zsh heap scope unless you duplicate into permanent storage.

## API ownership rules for module code

### `createparam(name, flags)`

`name` is an input key. If creation succeeds:

1. zsh allocates/uses a `Param` node;
2. zsh duplicates the name with `ztrdup(name)` for the hash table;
3. zsh assigns the standard GSU table for the type unless `PM_SPECIAL` is set;
4. the parameter has no value yet except zero/null defaults from `zshcalloc`.

If the name already exists in the current scope and cannot be recreated,
`createparam()` may return `NULL` while clearing `PM_UNSET` on the existing
node. A `NULL` return does not always mean “there is no such parameter”; zsh C
code often follows with `paramtab->getnode()`.

### `setsparam(name, value)` / `assignsparam(...)`

`name` is input; `value` is transferred to zsh on success or freed by zsh on
many failure paths.

`value` must be a zsh permanent string. For ordinary scalar params,
`strsetfn()` does:

```c
zsfree(pm->u.str);
pm->u.str = x;
```

So passing stack memory, Rust-owned memory, C literal memory, or `zhalloc()`
memory is wrong. Use `ztrdup()` for strings that zsh will own.

### `setiparam(name, zlong)` / `setnparam(name, mnumber)`

No string value is transferred for integer/float storage; zsh stores the numeric
payload in `pm->u.val` or `pm->u.dval` via the integer/float GSU. The name is
only a key. Arithmetic uses `mnumber` during evaluation, but normal integer
parameters store raw `zlong`.

### `setaparam(name, char **array)`

`array` is transferred to zsh. It must be a null-terminated permanent array,
and each element must be a permanent string. `arrsetfn()` eventually frees it
with `freearray()`, which frees each string and then the vector.

Use zsh helpers such as `mkarray(ztrdup(...))` for one element, `zarrdup()` for a
copy of an existing zsh array, or manually allocate the vector with `zalloc()`
and each element with `ztrdup()`.

Do not use `hmkarray()` or `zhalloc()` arrays for installed parameters.

### `sethparam(name, char **pairs)`

The `pairs` input is a flat key/value array. zsh creates a parameter table with
`newparamtable()`, creates scalar hash elements with `createparam()`, and gives
values to `assignstrvalue()`. Keys and values are consumed/freed/installed by
zsh. The pair array and strings therefore need the same permanent-allocation
care as arrays.

### `addparamdef(Paramdef d)`

Module feature parameters go through `module.c:addparamdef()`.

- `d->name` must stay valid in the `paramdef` table. Use static names.
- zsh still creates/looks up a real `Param` and stores it in `d->pm`.
- If `d->var` is supplied, zsh stores it in `pm->u.data`.
- If no custom GSU is supplied, zsh chooses `varscalar_gsu`, `varinteger_gsu`,
  or `vararray_gsu`, which dereference `pm->u.data` as a pointer to the backing
  variable.

That means `d->var` is not a boxed value that zsh frees. It is a pointer to
module-owned storage. In our Rust module this should be a `static mut` or other
module-lifetime object. Do not point it at a stack variable.

## Scope and lifetime

`runshfunc()` wraps function execution with `startparamscope()` and
`endparamscope()`.

- `startparamscope()` increments `locallevel`.
- `typeset`/`local` create locals by setting `PM_LOCAL` during creation and then
  recording `pm->level = locallevel`.
- If a local hides an outer parameter, the outer `Param` is removed from the
  visible table and linked from `pm->old`.
- `endparamscope()` decrements `locallevel` and scans `paramtab`.
- Any parameter whose `pm->level` is now deeper than `locallevel` is unset or
  restored.

Normal locals are removed with `unsetparam_pm(pm, ..., exp=0)`, which calls the
GSU unset function, removes the hash node, restores `pm->old` if present, and
frees the local node. Special parameters have additional restoration logic
because their visible node may be reused while old data is saved in a side copy.

So yes: a local associative array such as `mytable` is “dropped” at function
return in the zsh sense. Its hash table and element parameters are deleted by
zsh's parameter teardown, not by Rust or C stack unwinding.

## Line-by-line: the sample script

```zsh
typeset -a myarr=("scalar1" scalar2)
```

`typeset` maps `-a` to `PM_ARRAY`. Since this is top-level, there is no
function local scope, so it creates or reuses a global parameter. The node is a
`Param` allocated with `zshcalloc`; its table name is `ztrdup("myarr")`; its GSU
is `stdarray_gsu`. The two array elements are permanent zsh strings produced by
parsing/preforking the assignment and then transferred into the parameter with
`assignaparam()` / `setarrvalue()` / `arrsetfn()`.

```zsh
typeset -i myint=0
```

`-i` maps to `PM_INTEGER`. The parameter node is a normal integer parameter.
The value is stored as `pm->u.val` (`zlong`) via `stdinteger_gsu`. No separate
boxed heap object is allocated for the numeric value.

```zsh
mparam=''
```

A scalar assignment calls `setsparam()` / `assignsparam()`. If `mparam` does not
exist, zsh creates a `PM_SCALAR` parameter. The empty string value passed to the
GSU set function is a permanent zsh string, and `strsetfn()` stores it in
`pm->u.str`.

```zsh
if (($+SOME_ENV_VAR)); then
    typeset -i mparam
fi
```

`$+SOME_ENV_VAR` is a parameter-existence test evaluated in arithmetic context.
If true, `typeset -i mparam` attempts to change `mparam` from scalar to integer.
For ordinary non-special parameters, `typeset_single()` detects a type change,
unsets the old parameter, and creates a new integer parameter under the same
name. If no explicit value is supplied, zsh tries to carry over scalar value in
some scalar-to-scalar numeric cases; the final integer value is set through
numeric conversion rules. The resulting storage is `pm->u.val` (`zlong`), not a
string.

```zsh
myfn() {
```

Function definition is not a parameter. `execfuncdef()` allocates a `Shfunc`
with `zalloc()`, allocates or references a permanent `Eprog`, duplicates the
function name with `ztrdup()`, and inserts the node into `shfunctab`. It is a
hash-table entry analogous to parameters, but it is freed by the shell-function
hash table, not by `paramtab`.

```zsh
    local -A mytable=(
        [k]='hello'
        [v]='world'
    )
```

Inside function execution, `locallevel > 0`. `local` forces `PM_LOCAL`; `-A`
maps to `PM_HASHED`. zsh creates a local associative parameter with
`pm->level = locallevel`. The associative value is a parameter hash table made
with `newparamtable()`. Each key is a hash element parameter (`PM_HASHELEM`) and
each value is usually a scalar string. Keys and values are permanent zsh-owned
allocation. On function return, `endparamscope()` sees `mytable->level` is out
of scope, calls `unsetparam_pm(..., exp=0)`, and `hashsetfn()` /
`deleteparamtable()` delete the hash and its element params.

```zsh
    print -- "$@"
```

`$@` is special array parameter data backed by `pparams`, the function's
positional-parameter array. `doshfunc()` allocated `pparams` permanently for the
call frame with `zshcalloc()`/`ztrdup()` and frees it with `freearray()` before
returning from the function.

```zsh
    printf '%s: %s\n' ${(kv)mytable}
```

`${(kv)mytable}` expands the local associative array into alternating keys and
values. Expansion/scanning arrays are often `zhalloc()` temporaries because they
only need to survive through expansion and command invocation. That temporary
view is distinct from the permanent hash table stored in `mytable`.

```zsh
}

myfn hii
```

Calling `myfn` runs `execshfunc()`/`doshfunc()`/`runshfunc()`. zsh creates a
parameter scope, sets positional parameters, executes the function body, then
ends the parameter scope. At that point `mytable` is removed and freed/restored
as described above.

```zsh
unfunction myfn
```

`unfunction` is `bin_unhash()` against `shfunctab`. It removes the `Shfunc` hash
node and calls `freeshfuncnode()`, which frees the function program, redirection
program, filename/cache/sticky data, name, and `Shfunc` object. So yes: after
`unfunction`, the function definition is freed unless still protected by zsh's
reference-counting/in-use mechanisms.

```zsh
typeset -gE HAS_COLOR=1.2
```

`-E` selects an exponential float parameter (`PM_EFLOAT`). `-g` is a scope
choice, not an allocator choice. In a function, `-g` prevents `PM_LOCAL`; at top
level it is effectively already global. The parameter node and name are
permanent zsh allocations (`zshcalloc`, `ztrdup`). The value is stored inline as
`pm->u.dval` (`double`) through `stdfloat_gsu`; there is no boxed float string.

```zsh
for ((i = 0; i < 5; ++i)); do
```

Arithmetic `for` evaluates math expressions with `mnumber`. On assignment,
`setmathvar()` calls `setnparam()`. If `i` does not exist, zsh creates a numeric
parameter: usually `PM_INTEGER` for integer values, unless options force scalar
behavior. Once `i` is a normal integer parameter, its stored value is raw
`zlong` in `pm->u.val`. During expression evaluation, temporary results are
`mnumber` tagged values.

```zsh
    print -p "%B${i}%b"
done
```

`${i}` retrieves the parameter through its GSU and stringifies it for expansion.
Formatting/prompt escape expansion uses temporary heap strings; it does not
change the storage of `i`.

```zsh
echo $RANDOM
```

`RANDOM` is a special integer parameter from `special_params[]` with
`random_gsu`. Its getter calls `rand() & 0x7fff`; its setter calls `srand(v)`.
The `Param` entry exists in `paramtab`, but the value is not stored in
`pm->u.val`.

```zsh
RANDOM=$((${#commands[@]} + $SECONDS))
```

The RHS is arithmetic. `${#commands[@]}` counts elements of a special hash from
modules such as `zsh/parameter`; `$SECONDS` is another special parameter whose
integer getter computes elapsed time from shell timer state. The resulting
`mnumber`/`zlong` is assigned to `RANDOM`; `randomsetfn()` seeds the C PRNG.
Again, no normal integer storage in the `RANDOM` parameter itself.

```zsh
# tied variables
```

Tied variables connect a scalar and an array view, e.g. `PATH` and `path`, or
user-created `typeset -T scalar array [sep]`. zsh stores alternate names in
`pm->ename` and installs special GSU functions such as `colonarr_gsu` or
`tiedarr_gsu`. Setting the scalar splits into the array; setting the array joins
or updates the scalar/environment. User-created tied scalars allocate a
`struct tieddata` with `zalloc()` and free it in `tiedarrunsetfn()`. This is one
of the places where “boxed data” in `pm->u.data` really is an owned zsh object.

## Practical rules for Rust FFI in this repository

1. Static parameter names are preferred. They satisfy both zsh lifetime rules
   and our no-allocation preference.
2. For module `Paramdef` names, use static names; never `zhalloc`.
3. For `Paramdef.var`, use module-lifetime static backing storage. zsh will not
   free it.
4. For ordinary zsh-owned scalar/array/hash values, the zsh-correct answer is
   permanent allocation (`ztrdup`/`zalloc`/`newparamtable`), not `zhalloc`.
5. Any pointer returned by expansion/scanning/stringification helpers may be heap
   scoped. Duplicate it before storing it in a parameter; otherwise treat it as
   temporary.
6. Any `Param *` handle can become invalid after `unset`, `resetparam`, scope
   exit, type conversion, function return, module unload, or hash replacement.
   Prefer name-based operations and re-lookup after mutations.
