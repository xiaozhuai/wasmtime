mod format;
mod formatter;
mod inst;
mod operand;

use crate::dsl;
pub use formatter::Formatter;

/// Top-level function for generating Rust code.
pub fn generate(f: &mut Formatter, insts: &[dsl::Inst]) {
    // Generate "all instructions" enum.
    generate_inst_enum(f, insts);
    generate_inst_display_impl(f, insts);
    generate_inst_encode_impl(f, insts);

    // Generate per-instruction structs.
    f.empty_line();
    for inst in insts {
        inst.generate_struct(f);
        inst.generate_struct_impl(f);
        inst.generate_display_impl(f);
        f.empty_line();
    }
}

/// `enum Inst { ... }`
fn generate_inst_enum(f: &mut Formatter, insts: &[dsl::Inst]) {
    generate_derive(f);
    f.line("pub enum Inst {");
    f.indent_push();
    for inst in insts {
        inst.generate_enum_variant(f);
    }
    f.indent_pop();
    f.line("}");
}

/// `#[derive(...)]`
fn generate_derive(f: &mut Formatter) {
    f.line("#[derive(arbitrary::Arbitrary, Debug)]");
}

/// `impl std::fmt::Display for Inst { ... }`
fn generate_inst_display_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    f.line("impl std::fmt::Display for Inst {");
    f.indent(|f| {
        f.line("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        f.indent(|f| {
            f.line("match self {");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_display(f);
            }
            f.indent_pop();
            f.line("}");
        });
        f.line("}");
    });
    f.line("}");
}

/// `impl Inst { ... }`
fn generate_inst_encode_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    f.line("impl Inst {");
    f.indent(|f| {
        f.line("pub fn encode(&self, b: &mut MachBuffer<x64::Inst>) {");
        f.indent(|f| {
            f.line("match self {");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_encode(f);
            }
            f.indent_pop();
            f.line("}");
        });
        f.line("}");
    });
    f.line("}");
}
