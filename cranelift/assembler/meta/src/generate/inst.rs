use super::{fmtln, generate_derive, Formatter};
use crate::dsl::{self};

impl dsl::Inst {
    /// `struct <inst> { <op>: Reg, <op>: Reg, ... }`
    pub fn generate_struct(&self, f: &mut Formatter) {
        let struct_name = self.struct_name_with_generic();
        let struct_fields = self.generate_struct_fields();
        let where_clause = if self.requires_generic() {
            "where R: AsReg"
        } else {
            ""
        };

        f.line(format!("/// `{self}`"), None);
        generate_derive(f);
        fmtln!(f, "pub struct {struct_name} {where_clause} {{ {struct_fields} }}");
    }

    /// `<class name>_<format name>`
    #[must_use]
    fn struct_name(&self) -> String {
        format!("{}_{}", self.name.to_lowercase(), self.format.name.to_lowercase())
    }

    fn requires_generic(&self) -> bool {
        self.format.uses_arbitrary_register()
    }

    /// `<struct_name><R>`
    fn struct_name_with_generic(&self) -> String {
        let struct_name = self.struct_name();
        if self.requires_generic() {
            format!("{struct_name}<R>")
        } else {
            struct_name
        }
    }

    fn generate_impl_block_start(&self) -> &str {
        if self.requires_generic() {
            "impl<R: AsReg>"
        } else {
            "impl"
        }
    }

    /// `pub <op>: <type>, *`
    #[must_use]
    fn generate_struct_fields(&self) -> String {
        comma_join(
            self.format
                .operands_with_ty(Some("R"))
                .map(|(l, ty)| format!("pub {l}: {ty}")),
        )
    }

    /// `<inst>(<inst>),`
    pub fn generate_enum_variant(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        let struct_name = self.struct_name_with_generic();
        fmtln!(f, "{variant_name}({struct_name}),");
    }

    // `Self::<inst>(i) => write!(f, "{}", i),`
    pub fn generate_variant_display(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        fmtln!(f, "Self::{variant_name}(i) => write!(f, \"{{i}}\"),");
    }

    // `Self::<inst>(i) => i.encode(b, o),`
    pub fn generate_variant_encode(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        fmtln!(f, "Self::{variant_name}(i) => i.encode(b, o),");
    }

    // `Self::<inst>(i) => i.visit_operands(v),`
    pub fn generate_variant_regalloc(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        fmtln!(f, "Self::{variant_name}(i) => i.visit_operands(v),");
    }

    // `fn <inst>(<params>) -> Inst { ... }`
    pub fn generate_variant_constructor(&self, f: &mut Formatter) {
        let variant_name = self.struct_name();
        let params = comma_join(
            self.format
                .operands_with_ty(Some("R"))
                .map(|(l, ty)| format!("{l}: {ty}")),
        );
        let args = comma_join(
            self.format
                .operands_with_ty(None)
                .map(|(l, _)| format!("{l}")),
        );

        fmtln!(f, "pub fn {variant_name}<R: AsReg>({params}) -> Inst<R> {{");
        f.indent(|f| {
            fmtln!(f, "Inst::{variant_name}({variant_name} {{ {args} }})",);
        });
        fmtln!(f, "}}");
    }

    /// `impl <inst> { ... }`
    pub fn generate_struct_impl(&self, f: &mut Formatter) {
        let impl_block = self.generate_impl_block_start();
        let struct_name = self.struct_name_with_generic();
        fmtln!(f, "{impl_block} {struct_name} {{");

        f.indent_push();
        self.generate_encode_function(f);
        f.empty_line();
        self.generate_regalloc_function(f);
        f.indent_pop();
        fmtln!(f, "}}");
    }

    /// `fn encode(&self, buf: &mut impl CodeSink, off: &impl KnownOffsetTable) { ... }`
    pub fn generate_encode_function(&self, f: &mut Formatter) {
        let off = if self.format.uses_memory().is_some() {
            "off"
        } else {
            "_"
        };
        fmtln!(
            f,
            "pub fn encode(&self, buf: &mut impl CodeSink, {off}: &impl KnownOffsetTable) {{"
        );
        f.indent_push();

        // Emit trap.
        if let Some(op) = self.format.uses_memory() {
            f.empty_line();
            f.comment("Emit trap.");
            fmtln!(f, "if let GprMem::Mem({op}) = &self.{op} {{");
            f.indent(|f| {
                fmtln!(f, "if let Some(trap_code) = {op}.trap_code() {{");
                f.indent(|f| {
                    fmtln!(f, "buf.add_trap(trap_code);");
                });
                fmtln!(f, "}}");
            });
            fmtln!(f, "}}");
        }

        match &self.encoding {
            dsl::Encoding::Rex(rex) => self.format.generate_rex_encoding(f, rex),
            dsl::Encoding::Vex(_) => todo!(),
        }

        f.indent_pop();
        fmtln!(f, "}}");
    }

    /// `fn regalloc(&self) -> String { ... }`
    pub fn generate_regalloc_function(&self, f: &mut Formatter) {
        use dsl::OperandKind::*;
        let extra_generic_bound = if !self.requires_generic() {
            "<R: AsReg>"
        } else {
            ""
        };
        fmtln!(f, "pub fn visit_operands{extra_generic_bound}(&mut self, visitor: &mut impl OperandVisitor<R>) {{");
        f.indent(|f| {
            for o in &self.format.operands {
                match o.location.kind() {
                    Imm(_) => {
                        // Immediates do not need register allocation.
                    }
                    FixedReg(_) => {
                        let call = o.mutability.generate_regalloc_call();
                        let Some(fixed) = o.location.generate_fixed_reg() else {
                            unreachable!()
                        };
                        fmtln!(f, "visitor.fixed_{call}(&R::new({fixed}));");
                    }
                    Reg(reg) => {
                        let call = o.mutability.generate_regalloc_call();
                        fmtln!(f, "self.{reg}.{call}(visitor);");
                    }
                    RegMem(rm) => {
                        let call = o.mutability.generate_regalloc_call();
                        fmtln!(f, "self.{rm}.{call}(visitor);");
                    }
                }
            }
        });
        fmtln!(f, "}}");
    }

    /// `impl Display for <inst> { ... }`
    pub fn generate_display_impl(&self, f: &mut Formatter) {
        let impl_block = self.generate_impl_block_start();
        let struct_name = self.struct_name_with_generic();
        fmtln!(f, "{impl_block} std::fmt::Display for {struct_name} {{");
        f.indent_push();
        fmtln!(f, "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{");

        f.indent_push();
        for op in &self.format.operands {
            let location = op.location;
            let to_string = location.generate_to_string(op.extension);
            fmtln!(f, "let {location} = {to_string};");
        }

        let inst_name = &self.name;
        let ordered_ops = self.format.generate_att_style_operands();
        fmtln!(f, "write!(f, \"{inst_name} {ordered_ops}\")");
        f.indent_pop();
        fmtln!(f, "}}");

        f.indent_pop();
        fmtln!(f, "}}");
    }

    /// `fn x64_assemble_<inst>(&mut self, <params>) -> Inst<generic type> { ... }`
    pub fn generate_isle_macro(&self, f: &mut Formatter, generic_type_name: &str) {
        let inst_ty = format!("cranelift_assembler::Inst<{generic_type_name}>");
        let struct_name = self.struct_name();
        let operands = self
            .format
            .operands_with_ty(Some("PairedGpr"))
            .collect::<Vec<_>>();
        let params = comma_join(
            operands
                .iter()
                .map(|(l, ty)| format!("{l}: &cranelift_assembler::{ty}")),
        );
        let args = comma_join(operands.iter().map(|(l, _)| format!("{l}.clone()")));

        fmtln!(f, "fn x64_assemble_{struct_name}(&mut self, {params}) -> {inst_ty} {{",);
        f.indent(|f| {
            fmtln!(f, "cranelift_assembler::build::{struct_name}({args})",);
        });
        fmtln!(f, "}}");
    }

    /// `(decl x64_assemble_<inst> (<params>) AssemblerInst)
    ///  (extern constructor assemble_<inst> assemble_<inst>)`
    pub fn generate_isle_definition(&self, f: &mut Formatter) {
        let struct_name = self.struct_name();
        let rule_name = format!("x64_assemble_{struct_name}");
        let params = self
            .format
            .operands_with_ty(None)
            .map(|(_, ty)| format!("Assembler{ty}"))
            .collect::<Vec<_>>()
            .join(" ");

        f.line(format!("(decl {rule_name} ({params}) AssemblerInst)"), None);
        f.line(format!("(extern constructor {rule_name} {rule_name})"), None);
    }
}

fn comma_join<I: Into<String>>(items: impl Iterator<Item = I>) -> String {
    items.map(Into::into).collect::<Vec<_>>().join(", ")
}
