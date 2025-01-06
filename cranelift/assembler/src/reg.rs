//! Pure register operands.

use crate::{alloc::OperandVisitor, fuzz::FuzzReg, rex::RexFlags};
use arbitrary::Arbitrary;

pub mod enc {
    use super::Size;

    pub const RAX: u8 = 0;
    pub const RCX: u8 = 1;
    pub const RDX: u8 = 2;
    pub const RBX: u8 = 3;
    pub const RSP: u8 = 4;
    pub const RBP: u8 = 5;
    pub const RSI: u8 = 6;
    pub const RDI: u8 = 7;
    pub const R8: u8 = 8;
    pub const R9: u8 = 9;
    pub const R10: u8 = 10;
    pub const R11: u8 = 11;
    pub const R12: u8 = 12;
    pub const R13: u8 = 13;
    pub const R14: u8 = 14;
    pub const R15: u8 = 15;

    pub fn to_string(enc: u8, size: super::Size) -> &'static str {
        use Size::{Byte, Doubleword, Quadword, Word};
        match enc {
            RAX => match size {
                Byte => "%al",
                Word => "%ax",
                Doubleword => "%eax",
                Quadword => "%rax",
            },
            RBX => match size {
                Byte => "%bl",
                Word => "%bx",
                Doubleword => "%ebx",
                Quadword => "%rbx",
            },
            RCX => match size {
                Byte => "%cl",
                Word => "%cx",
                Doubleword => "%ecx",
                Quadword => "%rcx",
            },
            RDX => match size {
                Byte => "%dl",
                Word => "%dx",
                Doubleword => "%edx",
                Quadword => "%rdx",
            },
            RSI => match size {
                Byte => "%sil",
                Word => "%si",
                Doubleword => "%esi",
                Quadword => "%rsi",
            },
            RDI => match size {
                Byte => "%dil",
                Word => "%di",
                Doubleword => "%edi",
                Quadword => "%rdi",
            },
            RBP => match size {
                Byte => "%bpl",
                Word => "%bp",
                Doubleword => "%ebp",
                Quadword => "%rbp",
            },
            RSP => match size {
                Byte => "%spl",
                Word => "%sp",
                Doubleword => "%esp",
                Quadword => "%rsp",
            },
            R8 => match size {
                Byte => "%r8b",
                Word => "%r8w",
                Doubleword => "%r8d",
                Quadword => "%r8",
            },
            R9 => match size {
                Byte => "%r9b",
                Word => "%r9w",
                Doubleword => "%r9d",
                Quadword => "%r9",
            },
            R10 => match size {
                Byte => "%r10b",
                Word => "%r10w",
                Doubleword => "%r10d",
                Quadword => "%r10",
            },
            R11 => match size {
                Byte => "%r11b",
                Word => "%r11w",
                Doubleword => "%r11d",
                Quadword => "%r11",
            },
            R12 => match size {
                Byte => "%r12b",
                Word => "%r12w",
                Doubleword => "%r12d",
                Quadword => "%r12",
            },
            R13 => match size {
                Byte => "%r13b",
                Word => "%r13w",
                Doubleword => "%r13d",
                Quadword => "%r13",
            },
            R14 => match size {
                Byte => "%r14b",
                Word => "%r14w",
                Doubleword => "%r14d",
                Quadword => "%r14",
            },
            R15 => match size {
                Byte => "%r15b",
                Word => "%r15w",
                Doubleword => "%r15d",
                Quadword => "%r15",
            },
            _ => panic!("%invalid{}", enc), // TODO: print instead?
        }
    }
}

/// TODO
pub trait AsReg: Clone + std::fmt::Debug {
    // TODO: only useful for fuzz generation.
    fn new(enc: u8) -> Self;
    fn enc(&self) -> u8;
    //     fn to_string(&self, size: Size) -> &str;
}

impl AsReg for u8 {
    fn new(enc: u8) -> Self {
        enc
    }

    fn enc(&self) -> u8 {
        *self
    }

    // fn to_string(&self, size: Size) -> &str {
    //     todo!()
    // }
}

/// A general purpose x64 register (e.g., `%rax`).
///
/// This holds a larger value than needed to accommodate register allocation.
/// Cranelift's register allocator expects to modify an instruction's operands
/// in place (see [`Gpr::as_mut`]); Cranelift assigns a virtual register to each
/// operand and only later replaces these with true HW registers. A consequence:
/// register allocation _must happen_ before encoding the register (see
/// [`Gpr::enc`]).
#[derive(Clone, Copy, Debug)]
pub struct Gpr<R: AsReg = u8>(pub(crate) R);

impl<R: AsReg> Gpr<R> {
    /// Create a [`Gpr`] that may be real (emit-able in machine code) or virtual
    /// (waiting for register allocation).
    pub fn new(reg: R) -> Self {
        Self(reg)
    }

    pub fn enc(&self) -> u8 {
        let enc = self.0.enc();
        assert!(enc < 16, "invalid register: {}", enc);
        enc
    }

    pub fn to_string(&self, size: Size) -> &str {
        enc::to_string(self.enc(), size)
    }

    pub fn always_emit_if_8bit_needed(&self, rex: &mut RexFlags) {
        let enc_reg = self.0.enc();
        if (4..=7).contains(&enc_reg) {
            rex.always_emit();
        }
    }

    pub fn read(&mut self, visitor: &mut impl OperandVisitor<R>) {
        visitor.read(&mut self.0);
    }

    pub fn read_write(&mut self, visitor: &mut impl OperandVisitor<R>) {
        visitor.read_write(&mut self.0);
    }

    // /// Allow the register allocator to modify this register in place.
    // pub fn as_mut(&mut self) -> &mut u32 {
    //     &mut self
    // }

    // /// Allow external users to inspect this register.
    // pub fn as_u32(&self) -> u32 {
    //     self.0
    // }
}

impl<'a, R: AsReg> Arbitrary<'a> for Gpr<R> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let reg = FuzzReg::arbitrary(u)?;
        Ok(Self(R::new(reg.enc())))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Size {
    Byte,
    Word,
    Doubleword,
    Quadword,
}

#[derive(Clone, Debug)]
pub struct MinusRsp<R: AsReg>(R);

impl<R: AsReg> MinusRsp<R> {
    pub fn as_mut(&mut self) -> &mut R {
        &mut self.0
    }
    pub fn enc(&self) -> u8 {
        self.0.enc()
    }
}

// impl Arbitrary<'_> for MinusRsp<Gpr> {
//     fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
//         let gpr = u.choose(&[
//             RAX, RCX, RDX, RBX, RBP, RSI, RDI, R8, R9, R10,
//             R11, R12, R13, R14, R15,
//         ])?;
//         Ok(Self(Gpr(u32::from(*gpr))))
//     }
// }

impl<R: AsReg> Arbitrary<'_> for MinusRsp<R> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        use enc::*;
        let gpr = u.choose(&[
            RAX, RCX, RDX, RBX, RBP, RSI, RDI, R8, R9, R10, R11, R12, R13, R14, R15,
        ])?;
        Ok(Self(R::new(*gpr)))
    }
}
