use glob::glob;
use std::{env, path::{Path, PathBuf}};

fn main() {
    // Wrapper paths
    let wrapper_dir = Path::new("wrapper");
    let wrapper_src = wrapper_dir.join("suitesparse_wrapper.c");
    let wrapper_hdr = wrapper_dir.join("suitesparse_wrapper.h");
    // Suitesparse paths
    let suitesparse_dir = wrapper_dir.join("suitesparse");
    let cxsparse_src = suitesparse_dir.join("CXSparse/Source");
    let cxsparse_include = suitesparse_dir.join("CXSparse/Include");
    let suitesparse_config = suitesparse_dir.join("SuiteSparse_config");

    // Build SuiteSparse_config
    let mut config = cc::Build::new();
    config
        .file(suitesparse_config.join("SuiteSparse_config.c"))
        .include(&suitesparse_config);
    config.compile("suitesparse_config");

    // Build CXSparse
    let mut cxsparse = cc::Build::new();
    cxsparse
        .include(&cxsparse_include)
        .include(&suitesparse_config)
        .define("NCOMPLEX", None)
        .flag("-Wno-unused-parameter")
        .flag_if_supported("-std=c11");

    let patterns = [
        "cs_a*.c",
        "cs_chol*.c",
        "cs_compress.c",
        "cs_counts.c",
        "cs_cumsum.c",
        "cs_[d-z]*.c",
        "cxsparse_version.c",
    ];

    for pattern in patterns.iter() {
        let full_pattern = format!("{}/{pattern}", cxsparse_src.display());
        for entry in glob(&full_pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    cxsparse.file(path);
                }
                Err(e) => {
                    panic!("Glob error: {e}");
                }
            }
        }
    }
    cxsparse.compile("cxsparse");

    let mut wrapper = cc::Build::new();
    wrapper
        .file(&wrapper_src)
        .include(&cxsparse_include)
        .include(wrapper_dir)
        .include(&suitesparse_config);
    wrapper.compile("suitesparse_wrapper");

    println!("cargo:rustc-link-lib=static=cxsparse");
    println!("cargo:rustc-link-lib=static=suitesparse_config");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rerun-if-changed={}", wrapper_src.display());
    println!("cargo:rerun-if-changed={}", wrapper_hdr.display());

    let bindings = bindgen::Builder::default()
        .header(format!("{}", wrapper_hdr.display()))
        .clang_arg(format!("-I{}", cxsparse_include.display()))
        .clang_arg(format!("-I{}", suitesparse_config.display()))
        .allowlist_function("^(cs[ns]_init)$")
        .allowlist_function("^(csparse_.*)$")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("suitesparse.rs"))
        .expect("Couldn't write bindings!");
}
