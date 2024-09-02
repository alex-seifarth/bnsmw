# Library vsomeiprs

## Introduction 

*vsomeiprs* is a *Rust* ffi-wrapper library for the C++ based vsomeip.

## Source Layout

The following source directories are used:

- `vsomeipc`: This directory contains a C/C++ static library that *vsomeiprs* links to. The library provides a C interface for the C++ based *vsomeip* API. It is build by the `build.rs` script during the configuration phase which also generates the *Rust* ffi bindings.
- `src`: Contains the *Rust* API and its implementation of *vsomeiprs*.
- `build.rs`: Custom build script to build `vsomeipc` and generate the ffi bindings.

## Customized Location of *vsomeip*

The *vsomeipc* C/C++ library that *vsomeiprs* links to requires the *vomeip* library. The `CMakeList.txt` of *vsomeiprs* allos to spevify a custom location by having a `local.cmake` file either in this directory or one directory higher.

To set the custom location of *vsomeip* the CMAKE variable `vsomeip3_ROOT` must be set to the directory where `lib/cmake/vsomeipConfig` is found.

For example if an alternative installation of resource is done in `Documents/dev/usr`:
```bash 
# local.cmake
set(vsomeip3_ROOT "/home/<user>/Documents/dev/usr")
```