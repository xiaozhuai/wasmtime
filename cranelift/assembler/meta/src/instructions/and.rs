use crate::dsl::{fmt, inst, rex, Features::*, Inst, LegacyPrefixes::*, Operand::*};

pub fn list() -> Vec<Inst> {
    vec![
        inst("andb", fmt("I", [al, imm8]), rex(0x24).ib(), None),
        inst("andw", fmt("I", [ax, imm16]), rex(0x25).prefix(_66).iw(), None),
        inst("andl", fmt("I", [eax, imm32]), rex(0x25).id(), None),
        // inst("andq", fmt("I_SX", [rax, imm32]), rex(0x25).w().id(), None), // TODO: need a way to sign-extend the imm32, e.g., idsx()?
        inst("andb", fmt("MI", [rm8, imm8]), rex(0x80).digit(4).ib(), None),
        // inst("andb", fmt("MI_SX", [rm8, imm8]), rex(0x80).w().digit(4).ib(), None),
        inst("andw", fmt("MI", [rm16, imm16]), rex(0x81).prefix(_66).digit(4).iw(), None),
        inst("andl", fmt("MI", [rm32, imm32]), rex(0x81).digit(4).id(), None),
        // inst("andq", fmt("MI_SX", [rm64, imm32]), rex(0x81).w().digit(4).id(), None),
        // inst("andbw", fmt("MI_SX", [rm16, imm8]), rex(0x83).digit(4).ib(), None),
        // inst("andbd", fmt("MI_SX", [rm32, imm8]), rex(0x83).digit(4).ib(), None),
        // inst("andbq", fmt("MI_SX", [rm64, imm8]), rex(0x83).w().digit(4).ib(), None),
        // inst("andb", fmt("MR", [rm8, r8]), rex(0x20).r(), None),
        // inst("andb", fmt("MR_SX", [rm8, r8]), rex(0x20).w().r(), None),
        // inst("andw", fmt("MR", [rm16, r16]), rex(0x21).prefix(_66).r(), None),
        // inst("andl", fmt("MR", [rm32, r32]), rex(0x21).r(), None),
        // inst("andq", fmt("MR", [rm64, r64]), rex(0x21).w().r(), None),
        // inst("andb", fmt("RM", [r8, rm8]), rex(0x22).r(), None),
        // inst("andb", fmt("RM_SX", [r8, rm8]), rex(0x22).w().r(), None),
        // inst("andw", fmt("RM", [r16, rm16]), rex(0x23).prefix(_66).r(), None),
        // inst("andl", fmt("RM", [r32, rm32]), rex(0x23).r(), None),
        // inst("andq", fmt("RM", [r64, rm64]), rex(0x23).w().r(), None),
    ]
}
