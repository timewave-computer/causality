//! Build script for causality-ffi.
//! This script is now minimal and primarily handles C binding generation
//! when the `c-ffi` feature is enabled. OCaml-specific linking has been removed.

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Generate C bindings if cbindgen feature is enabled
    if cfg!(feature = "c-ffi") {
        generate_c_bindings();
    }
}

fn generate_c_bindings() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join("include")
        .join(format!("{}.h", package_name));

    // Ensure the parent directory exists.
    if let Some(parent_dir) = output_file.parent() {
        if let Err(e) = std::fs::create_dir_all(parent_dir) {
            eprintln!(
                "Warning: Unable to create directory {}: {}",
                parent_dir.display(),
                e
            );
            return;
        }
    }

    match cbindgen::generate(&*crate_dir) {
        Ok(bindings) => {
            if !bindings.write_to_file(&output_file) {
                eprintln!(
                    "Warning: Unable to write C bindings to {}",
                    output_file.display()
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: Unable to generate C bindings: {}", e);
        }
    }
}

fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        // Fallback for cases where CARGO_TARGET_DIR is not set
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("../../../")
    }
}
