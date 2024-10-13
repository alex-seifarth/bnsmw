// SPDX-License-Identifier: MPL-2.0
//
// Copyright (C) 2024 Alexander Seifarth
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{env, fs};
use std::path::{Path, PathBuf};
use cmake;

fn main() { 
    // build the vsomeipc library (static) that wraps the vsomeip3 lib
    let dst_vsomeipc = cmake::build("vsomeipc").join("lib");
    println!("cargo:rustc-link-search=native={}", dst_vsomeipc.display());
    println!("cargo:rustc-link-lib=static=vsomeipc");

    // The CMAKE build of vsomeipc creates a text file lib-locations.txt with the locations
    // of the dynamic libraries we must link against.
    // This file consist of lines of the form:  <lib-name>:<lib-location-path>
    let lib_locations = fs::read_to_string(dst_vsomeipc.join("lib-locations.txt"))
        .expect("Unable to read lib-locations.txt file.");
    lib_locations.lines()
        .map(|line| line.trim())
        .filter( |line| !line.is_empty())
        .for_each( | line | {
            let components: Vec<&str> = line.split(':').collect();
            assert_eq!(components.len(), 2, "lib-location invalid format");
            let lib_name = components[0].trim();
            let path = Path::new(components[1]).parent()
                .expect("Invalid path name for the library location {line}");
            println!("cargo:rustc-link-lib=dylib={lib_name}");
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    );

    println!("cargo::rerun-if-changed=vsomeipc/vsomeipc.h");
    println!("cargo::rerun-if-changed=vsomeipc/vsomeipc.cpp");
    println!("cargo::rerun-if-changed=vsomeipc/application.h");
    println!("cargo::rerun-if-changed=vsomeipc/application.cpp");
    println!("cargo::rerun-if-changed=vsomeipc/CMakeLists.txt");

    // we're linking C++ libraris - so we need the C++ std library.
    // TODO: windows?
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-flags=-l dylib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-flags=-l dylib=stdc++");
    }

    // Tell cargo to look for shared libraries in the specified directory
    //println!("cargo:rustc-link-search=/path/to/lib");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("vsomeipc/vsomeipc.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
