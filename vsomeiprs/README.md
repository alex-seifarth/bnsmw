# Library vsomeiprs

## Introduction 

*vsomeiprs* is a *Rust* ffi-wrapper library for the C++ based vsomeip.

## Building

### Requirements

*vsomeiprs* is actually only tested and supported on Debian based Linux operating system.
It requires
* system C/C++ compiler and linker (gcc-13),
* clang library (>= 16),
* cmake (>= 3.28),
* vsomeip (v3.5) (headers, shared libraries, cmake-config),
* Rust / Cargo (tested with 1.81).

The first three requirements can be installed on Debian based Linux system with
```bash
sudo apt update
sudo apt install build-essential clang-18 cmake
```

To build and install vsomeip see https://github.com/COVESA/vsomeip.

To install the Rust toolchain see https://www.rust-lang.org/tools/install.

### Building

Checkout this repository and build with cargo:
```bash
git clone https://github.com/alex-seifarth/bnsmw.git
cd bnsmw
cargo build
```

### Testing

To run the unit and integration tests
```bash
cargo test
```
Note that the integration tests run vsomeip with internal services (without network setup).
They will fail if some something prevents the test cases to run vsomeip in this way. For 
instance an already running vsomeip application or existing vsomeip configuration on the host
may prevent successful execution of the tests. It is therefore recommended to run tests inside 
a clean container.

To reduce the clutter of vsomeip logging message on the console there is a `vsomeip.json` configuration file under the package directory that disables vsomeip console logging. If these logging messages are desired for analysis then change the following in `vsomeip.json`:
```bash
# ./vsomeip.json
{
  "logging": {
    "level": "fatal",       # <- change to debug or info to see all messages
    "console": "false",     # <- change to true
    "file": {
      "enable": "false",
      "path": "/tmp/vsomeip.log"
    },
    "dlt": "false"
  }
}
```


### Customized Location and Version of *vsomeip*

The *vsomeipc* C/C++ library that *vsomeiprs* links to requires the *vomeip* library. The `CMakeList.txt` of *vsomeiprs* allows specifying a custom location by having a `local.cmake` file either in this directory or one directory higher.

To set the custom location of *vsomeip* the CMAKE variable `vsomeip3_ROOT` must be set to the directory where `lib/cmake/vsomeipConfig` is found.

For example if an alternative installation of resource is done in `Documents/dev/usr`:
```bash 
# local.cmake
set(vsomeip3_ROOT "/home/<user>/Documents/dev/usr")
```
Similarly, it is possible to specify a custom version
for vsomeip. In this case the variable `vsomeip_VERSION` must be set in the `local.cmake` file, for example:
```bash 
# local.cmake
set(vsomeip_VERSION "3.4")
```


## Internals

### Source Layout

The following source directories are used:

- `vsomeipc`: This directory contains a C/C++ static library that *vsomeiprs* links to. The library provides a C interface for the C++ based *vsomeip* API. It is build by the `build.rs` script during the configuration phase which also generates the *Rust* ffi bindings.
- `src`: Contains the *Rust* API and its implementation of *vsomeiprs*.
- `build.rs`: Custom build script to build `vsomeipc` and generate the ffi bindings.

