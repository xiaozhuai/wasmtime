mod features;
mod format;
mod formatter;
mod inst;
mod operand;

use crate::dsl;
use formatter::fmtln;
pub use formatter::Formatter;

/// Generate the Rust assembler code; e.g., `enum Inst { ... }`.
pub fn rust_assembler(f: &mut Formatter, insts: &[dsl::Inst]) {
    // Generate "all instructions" enum.
    generate_inst_enum(f, insts);

    generate_inst_display_impl(f, insts);
    generate_inst_encode_impl(f, insts);
    generate_inst_visit_impl(f, insts);
    generate_inst_match_impl(f, insts);
    generate_inst_constructor_impl(f, insts);

    // Generate per-instruction structs.
    f.empty_line();
    for inst in insts {
        inst.generate_struct(f);
        inst.generate_struct_impl(f);
        inst.generate_display_impl(f);
        f.empty_line();
    }

    dsl::Flag::generate_enum(f);
}

/// Generate the `isle_assembler_methods!` macro.
pub fn isle_macro(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "#[macro_export]");
    fmtln!(f, "macro_rules! isle_assembler_methods {{");
    f.indent(|f| {
        fmtln!(f, "() => {{");
        f.indent(|f| {
            for inst in insts {
                inst.generate_isle_macro(f, "Gpr", "PairedGpr");
            }
        });
        fmtln!(f, "}};");
    });
    fmtln!(f, "}}");
}

/// Generate the ISLE definitions that match the `isle_assembler_methods!` macro
/// above.
pub fn isle_definitions(f: &mut Formatter, insts: &[dsl::Inst]) {
    f.line("(type AssemblerImm8 extern (enum))", None);
    f.line("(type AssemblerImm16 extern (enum))", None);
    f.line("(type AssemblerImm32 extern (enum))", None);
    f.line("(type AssemblerReadGpr extern (enum))", None);
    f.line("(type AssemblerReadWriteGpr extern (enum))", None);
    f.line("(type AssemblerReadGprMem extern (enum))", None);
    f.line("(type AssemblerReadWriteGprMem extern (enum))", None);
    f.line("(type AssemblerInst extern (enum))", None);
    f.empty_line();
    for inst in insts {
        inst.generate_isle_definition(f);
        f.empty_line();
    }
}

/// `enum Inst { ... }`
fn generate_inst_enum(f: &mut Formatter, insts: &[dsl::Inst]) {
    generate_derive(f);
    fmtln!(f, "pub enum Inst<R: Registers> {{");
    f.indent_push();
    for inst in insts {
        inst.generate_enum_variant(f);
    }
    f.indent_pop();
    fmtln!(f, "}}");
}

/// `#[derive(...)]`
fn generate_derive(f: &mut Formatter) {
    f.line("#[derive(arbitrary::Arbitrary, Clone, Debug)]", None);
}

/// `impl std::fmt::Display for Inst { ... }`
fn generate_inst_display_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "impl<R: Registers> std::fmt::Display for Inst<R> {{");
    f.indent(|f| {
        fmtln!(f, "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{");
        f.indent(|f| {
            fmtln!(f, "match self {{");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_display(f);
            }
            f.indent_pop();
            fmtln!(f, "}}");
        });
        fmtln!(f, "}}");
    });
    fmtln!(f, "}}");
}

/// `impl Inst { fn encode... }`
fn generate_inst_encode_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "impl<R: Registers> Inst<R> {{");
    f.indent(|f| {
        fmtln!(f, "pub fn encode(&self, b: &mut impl CodeSink, o: &impl KnownOffsetTable) {{");
        f.indent(|f| {
            fmtln!(f, "match self {{");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_encode(f);
            }
            f.indent_pop();
            fmtln!(f, "}}");
        });
        fmtln!(f, "}}");
    });
    fmtln!(f, "}}");
}

/// `impl Inst { fn visit_operands... }`
fn generate_inst_visit_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "impl<R: Registers> Inst<R> {{");
    f.indent(|f| {
        fmtln!(f, "pub fn visit_operands(&mut self, v: &mut impl OperandVisitor<R>) {{");
        f.indent(|f| {
            fmtln!(f, "match self {{");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_visit(f);
            }
            f.indent_pop();
            fmtln!(f, "}}");
        });
        fmtln!(f, "}}");
    });
    fmtln!(f, "}}");
}

/// `impl Inst { fn match_features... }`
fn generate_inst_match_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "impl<R: Registers> Inst<R> {{");
    f.indent(|f| {
        fmtln!(f, "pub fn match_features(&self, f: AvailableFeatures) -> bool {{");
        f.indent(|f| {
            fmtln!(f, "match self {{");
            f.indent_push();
            for inst in insts {
                inst.generate_variant_match(f);
            }
            f.indent_pop();
            fmtln!(f, "}}");
        });
        fmtln!(f, "}}");
    });
    fmtln!(f, "}}");
}

/// `pub mod build { pub fn <inst>... }`
fn generate_inst_constructor_impl(f: &mut Formatter, insts: &[dsl::Inst]) {
    fmtln!(f, "pub mod build {{");
    f.indent(|f| {
        fmtln!(f, "use super::*;");
        for inst in insts {
            inst.generate_variant_constructor(f);
        }
    });
    fmtln!(f, "}}");
}

pub fn maybe_file_loc(fmtstr: &str, file: &'static str, line: u32) -> Option<FileLocation> {
    if fmtstr.ends_with(['{', '}']) {
        None
    } else {
        Some(FileLocation { file, line })
    }
}

pub struct FileLocation {
    pub(crate) file: &'static str,
    pub(crate) line: u32,
}

impl FileLocation {
    /// ```
    /// FileLocation::new(file!(), line!());
    /// ```
    pub fn new(file: &'static str, line: u32) -> Self {
        Self { file, line }
    }
}

impl core::fmt::Display for FileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}
