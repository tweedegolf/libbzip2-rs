# `libbzip2-rs-sys-cdylib`

A drop-in replacement for the `libbz2` dynamic library

```sh
# build the cdylib
# using `cargo build` will work but has limitations, see below
cargo build --release

# the extension of a cdylib varies per platform
cc bzpipe.c -o bzpipe target/release/libbz2_rs.so -I ../

# verify the implementation can compress and decompress our Cargo.toml
./bzpipe < Cargo.toml | ./bzpipe -d
```

By default this build uses libc `malloc`/`free` to (de)allocate memory, and only depends on the rust `core` library.
See below for the available feature flags.

## Feature Flags

### Allocators

We provide three options for the default allocator

**`c-allocator`**

```sh
cargo build --release --no-default-features --features "c-allocator"
```

Uses the libc `malloc` and `free` functions for memory allocation.

**`rust-allocator`**

```sh
cargo build --release --no-default-features --features "std,rust-allocator"
```
Uses the rust standard library global allocator for memory allocation.

**no allocator**

```sh
cargo build --release --no-default-features
```

No allocator is configured automatically. This means that, before [`BZ2_bzCompressInit`] or [`BZ2_bzDecompressInit`] are called,
the user must set the `bzalloc` and `bzfree` fields of the `bz_stream` to valid allocation and deallocation functions,
and the `opaque` field to either `NULL` or a pointer expected by the (de)allocation functions.

If no allocator is configured, the initialization functions will return `BZ_PARAM_ERROR`.

### Symbol Prefix

Symbols in C programs all live in the same namespace. A common solution to prevent names from clashing is to prefix
all of a library's symbols with a prefix. We support prefixing the name at build time with the `custom-prefix` feature
flag. When enabled, the value of the `LIBBZIP2_RS_SYS_PREFIX` is used as a prefix for all exported symbols. For example:

```ignore
> LIBBZIP2_RS_SYS_PREFIX="MY_CUSTOM_PREFIX_" cargo build --release --features=custom-prefix

   Compiling libbzip2-rs-sys v0.0.0 (libbzip2-rs/libbzip2-rs-sys)
   Compiling libz-rs-sys-cdylib v0.0.0 (libbzip2-rs/libbzip2-rs-sys-cdylib)
    Finished `release` profile [optimized] target(s) in 0.97s
> objdump -tT target/release/libbz2_rs.so | grep "BZ2_bzCompressInit"
000000000002f300 l     F .text	0000000000000441              .hidden _ZN15libbzip2_rs_sys5bzlib22BZ2_bzCompressInitHelp17hac60bda3d983fe05E
000000000002f2e0 g     F .text	000000000000001a              MY_CUSTOM_PREFIX_BZ2_bzCompressInit
000000000002f2e0 g    DF .text	000000000000001a  Base        MY_CUSTOM_PREFIX_BZ2_bzCompressInit
```

### `![no_std]`

The dynamic library can be built without the rust `std` crate, e.g. for embedded devices that don't support it. Disabling
the standard library has the following limitations:

- The `rust-allocator` should not be used. It internally enables the standard library, causing issues. Using `c-allocator`
    or not providing an allocator at build time is still supported.On embedded it is most common to provide a custom allocator
    that "allocates" into a custom array.

## Build for Distribution

A `cargo build` currently does not set some fields that are required or useful when using a dynamic library from C.
For instance, the soname and version are not set by a standard `cargo build`.

To build a proper, installable dynamic library, we recommend [`cargo-c`](https://github.com/lu-zero/cargo-c):

```
cargo install cargo-c
```

This tool deals with setting fields (soname, version) that a normal `cargo build` does not set (today).
It's configuration is in the `Cargo.toml`, where e.g. the library name or version can be changed.

```
> cargo cbuild --release
   Compiling libc v0.2.167
   Compiling libbzip2-rs-sys v0.0.0 (libbzip2-rs/libbzip2-rs-sys)
   Compiling libz-rs-sys-cdylib v0.0.0 (libbzip2-rs/libbzip2-rs-sys-cdylib)
    Finished `release` profile [optimized] target(s) in 1.63s
    Building pkg-config files
> tree target
target
├── CACHEDIR.TAG
└── x86_64-unknown-linux-gnu
    ├── CACHEDIR.TAG
    └── release
        ├── examples
        ├── incremental
        ├── libbz2_rs.a
        ├── libbz2_rs.d
        ├── libbz2_rs.pc
        ├── libbz2_rs.so
        └── libbz2_rs-uninstalled.pc
```
