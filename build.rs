//! Builds the sleef library from source.
extern crate cmake;
extern crate bindgen;

use std::{path::PathBuf, env};

fn main() {
    let target = env::var("TARGET").expect("TARGET was not set");

    // Parse target features, this is required for ABI compatibility.
    let mut features = std::collections::HashSet::<String>::default();
    if let Ok(rustflags) = env::var("CARGO_CFG_TARGET_FEATURE") {
        for v in rustflags.split(',') {
            features.insert(v.to_string());
        }
    }
    eprintln!("features: {:?}", features);

    let dst = cmake::Config::new("sleef")
        // .very_verbose(true)
        // no DFT libraries (should be behind a feature flag):
        .define("BUILD_DFT", "FALSE")
        // no tests (should build and run the tests behind a feature flag):
        .define("BUILD_TESTS", "FALSE")
        .define("BUILD_SHARED_LIBS", "TRUE")
        .build();

    println!("cargo:rustc-link-lib=sleef");
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR was not set"));
    let sleef_header = out_dir.join("include").join("sleef.h");
    assert!(sleef_header.exists(),
            "error sleef.h header not found in OUT_DIR={}",
            out_dir.display());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
        .header(sleef_header.to_str().expect("failed to convert header path to string"))
    // Rust does not support 80-bit precision floats:
        .opaque_type("Sleef_longdouble2")
    // The bindings should be no_std:
        .use_core()
    // The bindings should use the ctypes from libc, not std::os::raw:
        .ctypes_prefix("::libc")
    // Generate inline functions:
        .generate_inline_functions(true)
    // Only target nightly Rust for the time being:
        .unstable_rust(true)
        .rust_target(bindgen::RustTarget::Nightly)
    // Blacklist architecture specific vector types
        .blacklist_type("__m128d")
        .opaque_type("__m128d")
    // Finish the builder and generate the bindings.
        .generate()
    // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=sleef");
}
