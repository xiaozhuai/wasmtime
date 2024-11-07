//! Print the path to the generated code.

fn main() {
    let path = concat!(env!("OUT_DIR"), "/assembler.rs");
    println!("{path}");
}
