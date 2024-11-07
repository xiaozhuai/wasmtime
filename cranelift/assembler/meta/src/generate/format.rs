use super::Formatter;
use crate::dsl;

impl dsl::Format {
    #[must_use]
    pub fn generate_att_style_operands(&self) -> String {
        let mut ordered_ops: Vec<_> = self.operands.iter().map(|o| format!("{{{o}}}")).collect();
        if ordered_ops.len() > 1 {
            let first = ordered_ops.remove(0);
            ordered_ops.push(first);
        }
        ordered_ops.join(", ")
    }

    pub fn generate_rex_encoding(&self, f: &mut Formatter, rex: &dsl::Rex) {
        // Emit legacy prefixes.
        if rex.prefixes != dsl::LegacyPrefixes::NoPrefix {
            f.comment("Emit legacy prefixes.");
            match rex.prefixes {
                dsl::LegacyPrefixes::NoPrefix => unreachable!(),
                dsl::LegacyPrefixes::_66 => f.line("buf.put1(0x66);"),
                dsl::LegacyPrefixes::_F0 => f.line("buf.put1(0xf0);"),
                dsl::LegacyPrefixes::_66F0 => f.line("buf.put1(0x66); buf.put1(0xf0);"),
                dsl::LegacyPrefixes::_F2 => f.line("buf.put1(0xf2);"),
                dsl::LegacyPrefixes::_F3 => f.line("buf.put1(0xf3);"),
                dsl::LegacyPrefixes::_66F3 => f.line("buf.put1(0x66); buf.put1(0xf3"),
            }
        }

        // Emit REX prefix.
        f.comment("Emit REX prefix.");
        self.generate_rex_prefix(f, rex);

        // Emit opcodes.
        f.comment("Emit opcode.");
        f.line(format!("buf.put1(0x{:x});", rex.opcode));

        // Emit ModR/M byte.
        f.comment("Emit ModR/M byte.");
        self.generate_modrm_byte(f, rex);

        // Emit immediates.
    }

    fn generate_rex_prefix(&self, f: &mut Formatter, rex: &dsl::Rex) {
        use dsl::OperandKind::{FixedReg, Imm, Reg, RegMem};

        match self.operands.as_slice() {
            [dsl::Operand::r8, _] => {
                f.line("let mut rex = RexFlags::clear_w();");
                f.line("self.r8.always_emit_if_8bit_needed(&mut rex);");
            }
            [dsl::Operand::rm8, _] => {
                f.line("let mut rex = RexFlags::clear_w();");
                f.line("match &self.rm8 {");
                f.indent(|f| {
                    f.line("GprMem::Gpr(rm8) => { rm8.always_emit_if_8bit_needed(&mut rex); }");
                    f.line("GprMem::Mem(_) => {}");
                });
                f.line("}");
            }
            _ => {
                if rex.w {
                    f.line("let rex = RexFlags::set_w();");
                } else {
                    f.line("let rex = RexFlags::clear_w();");
                }
            }
        }

        match self.operands_by_kind().as_slice() {
            [FixedReg(dst), Imm(_)] => {
                // TODO: don't emit REX here
                f.line(format!("let {dst} = {};", dst.generate_fixed_reg().unwrap()));
                f.line(format!("let digit = 0x{:x};", rex.digit));
                f.line(format!("rex.emit_two_op(buf, digit, {dst}.enc());"));
            }
            [RegMem(dst), Imm(_)] => {
                if rex.digit > 0 {
                    f.line(format!("let digit = 0x{:x};", rex.digit));
                    f.line(format!("match &self.{dst} {{"));
                    f.indent(|f| {
                        f.line(format!(
                            "GprMem::Gpr({dst}) => rex.emit_two_op(buf, digit, {dst}.enc()),"
                        ));
                        f.line(format!(
                            "GprMem::Mem({dst}) => {dst}.emit_rex_prefix(rex, digit, buf),"
                        ));
                    });
                    f.line("}");
                } else {
                    todo!();
                }
            }
            [Reg(dst), RegMem(src)] => {
                f.line(format!("let {dst} = self.{dst}.enc();"));
                f.line(format!("match &self.{src} {{"));
                f.indent(|f| {
                    f.line(format!(
                        "GprMem::Gpr({src}) => rex.emit_two_op(buf, {dst}, {src}.enc()),"
                    ));
                    f.line(format!(
                        "GprMem::Mem({src}) => {src}.emit_rex_prefix(rex, {dst}, buf),"
                    ));
                });
                f.line("}");
            }
            [RegMem(dst), Reg(src)] => {
                f.line(format!("let {src} = self.{src}.enc();"));
                f.line(format!("match &self.{dst} {{"));
                f.indent(|f| {
                    f.line(format!(
                        "GprMem::Gpr({dst}) => rex.emit_two_op(buf, {dst}.enc(), {src}),"
                    ));
                    f.line(format!(
                        "GprMem::Mem({dst}) => {dst}.emit_rex_prefix(rex, {src}, buf),"
                    ));
                });
                f.line("}");
            }

            unknown => todo!("unknown pattern: {unknown:?}"),
        }
    }

    fn generate_modrm_byte(&self, f: &mut Formatter, rex: &dsl::Rex) {
        use dsl::OperandKind::{FixedReg, Imm, Reg, RegMem};
        match self.operands_by_kind().as_slice() {
            [FixedReg(_), Imm(imm)] => {
                f.line(format!("let bytes = {};", imm.bytes()));
                f.line(format!("let value = self.{imm}.value() as u32;"));
                f.line("emit_simm(buf, bytes, value);");
            }
            [RegMem(dst), Imm(imm)] => {
                debug_assert!(rex.digit > 0);
                f.line(format!("let digit = 0x{:x};", rex.digit));
                f.line(format!("match &self.{dst} {{"));
                f.indent(|f| {
                    f.line(format!("GprMem::Gpr({dst}) => emit_modrm(buf, digit, {dst}.enc()),"));
                    f.line(format!(
                        "GprMem::Mem({dst}) => emit_modrm_sib_disp(buf, digit, {dst}, 0, None),"
                    ));
                });
                f.line("}");
                f.line(format!("let bytes = {};", imm.bytes()));
                f.line(format!("let value = self.{imm}.value() as u32;"));
                f.line("emit_simm(buf, bytes, value);");
            }
            [Reg(dst), RegMem(src)] => {
                f.line(format!("let {dst} = self.{dst}.enc();"));
                f.line(format!("match &self.{src} {{"));
                f.indent(|f| {
                    f.line(format!("GprMem::Gpr({src}) => emit_modrm(buf, {dst}, {src}.enc()),"));
                    f.line(format!(
                        "GprMem::Mem({src}) => emit_modrm_sib_disp(buf, {dst}, {src}, 0, None),"
                    ));
                });
                f.line("}");
            }
            [RegMem(dst), Reg(src)] => {
                f.line(format!("let {src} = self.{src}.enc();"));
                f.line(format!("match &self.{dst} {{"));
                f.indent(|f| {
                    f.line(format!("GprMem::Gpr({dst}) => emit_modrm(buf, {src}, {dst}.enc()),"));
                    f.line(format!(
                        "GprMem::Mem({dst}) => emit_modrm_sib_disp(buf, {src}, {dst}, 0, None),"
                    ));
                });
                f.line("}");
            }

            unknown => todo!("unknown pattern: {unknown:?}"),
        }
    }
}
