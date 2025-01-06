use cranelift_assembler_meta as meta;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("The OUT_DIR environment variable must be set");
    let out_dir = Path::new(&out_dir);
    let built_files = vec![
        meta::generate_rust_assembler(&out_dir.join("assembler.rs")),
        meta::generate_isle_macro(&out_dir.join("assembler-isle-macro.rs")),
        meta::generate_isle_definitions(&out_dir.join("assembler-definitions.isle")),
    ];

    println!(
        "cargo:rustc-env=ASSEMBLER_BUILT_FILES={}",
        built_files.iter().map(print).collect::<Vec<_>>().join(":")
    );
}

fn print(p: &PathBuf) -> String {
    p.as_path().to_string_lossy().to_string()
}
