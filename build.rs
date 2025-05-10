use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::builder()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.hpp")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Enable C++ namespaces.
        .enable_cxx_namespaces()
        .respect_cxx_access_specs(true)
        
        // Explicitly allow functions and types we want
        .allowlist_item("rgbcx::.*")
        
        // Generation Settings
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .impl_debug(true)
        .impl_partialeq(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .rustified_enum(".*")
        
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Unable to write bindings");

    cc::Build::new()
        .cpp(true)
        .include("bc7enc_rdo/")
        .files(&["bc7enc_rdo/rgbcx.cpp"])
        .compile("bc7enc");
}
