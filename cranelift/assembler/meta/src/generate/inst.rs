use super::{generate_derive, Formatter};
use crate::dsl;

impl dsl::Inst {
    /// `<class name>_<format name>`
    #[must_use]
    pub fn struct_name(&self) -> String {
        format!("{}_{}", self.name.to_lowercase(), self.format.name.to_lowercase())
    }

    /// `<inst>(<inst>),`
    pub fn generate_enum_variant(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        let struct_name = self.struct_name();
        f.line(format!("{variant_name}({struct_name}),"));
    }

    // `Self::<inst>(i) => write!(f, "{}", i),`
    pub fn generate_variant_display(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        f.line(format!("Self::{variant_name}(i) => write!(f, \"{{}}\", i),"));
    }

    // `Self::<inst>(i) => i.encode(b),`
    pub fn generate_variant_encode(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        f.line(format!("Self::{variant_name}(i) => i.encode(b),"));
    }

    /// `struct <inst> { <op>: Reg, <op>: Reg, ... }`
    pub fn generate_struct(&self, f: &mut Formatter) {
        let struct_name = self.struct_name();
        let struct_fields = self
            .format
            .operands
            .iter()
            .filter_map(|op| {
                let ty = op.generate_type()?;
                Some(format!("pub {op}: {ty}"))
            })
            .collect::<Vec<String>>()
            .join(", ");
        generate_derive(f);
        f.line(format!("pub struct {struct_name} {{ {struct_fields} }}"));
    }

    /// `impl <inst> { ... }`
    pub fn generate_struct_impl(&self, f: &mut Formatter) {
        let struct_name = self.struct_name();
        f.line(format!("impl {struct_name} {{"));
        f.indent_push();
        self.generate_encode_function(f);
        f.empty_line();
        self.generate_regalloc_function(f);
        f.indent_pop();
        f.line("}");
    }

    /// `fn encode(&self, buf: &mut Vec<u8>) { ... }`
    pub fn generate_encode_function(&self, f: &mut Formatter) {
        f.line("pub fn encode(&self, buf: &mut MachBuffer<x64::Inst>) {");
        f.indent_push();

        // Emit trap.
        if let Some(op) = self.format.uses_memory() {
            f.comment("Emit trap.");
            f.line(format!("if let GprMem::Mem({op}) = &self.{op} {{"));
            f.indent(|f| {
                f.line(format!("if let Some(trap_code) = {op}.get_flags().trap_code() {{"));
                f.indent(|f| {
                    f.line("buf.add_trap(trap_code);");
                });
                f.line("}");
            });
            f.line("}");
        }

        match &self.encoding {
            dsl::Encoding::Rex(rex) => self.format.generate_rex_encoding(f, rex),
            dsl::Encoding::Vex(_) => todo!(),
        }

        f.indent_pop();
        f.line("}");
    }

    /// `fn regalloc(&self) -> String { ... }`
    pub fn generate_regalloc_function(&self, f: &mut Formatter) {
        f.line("pub fn regalloc(&self) {");
        f.indent_push();
        f.line("todo!();");
        f.indent_pop();
        f.line("}");
    }

    /// `impl Debug for <inst> { ... }`
    pub fn generate_display_impl(&self, f: &mut Formatter) {
        let struct_name = self.struct_name();
        f.line(format!("impl std::fmt::Display for {struct_name} {{"));
        f.indent_push();
        f.line("fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {");

        f.indent_push();
        for op in &self.format.operands {
            let to_string = op.generate_to_string();
            f.line(format!("let {op} = {to_string};"));
        }
        let inst_name = &self.name;
        let ordered_ops = self.format.generate_att_style_operands();
        f.line(format!("write!(f, \"{inst_name} {ordered_ops}\")"));
        f.indent_pop();
        f.line("}");

        f.indent_pop();
        f.line("}");
    }
}
