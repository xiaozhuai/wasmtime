use cranelift_assembler_meta as meta;
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var("OUT_DIR").expect("The OUT_DIR environment variable must be set");
    let assembler_file = Path::new(&out_dir).join("assembler.rs");
    meta::generate(&assembler_file);
}
