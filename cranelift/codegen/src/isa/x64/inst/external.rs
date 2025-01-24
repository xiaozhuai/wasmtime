//! Interface with the external assembler crate.

use crate::ir::TrapCode;

use super::{regs, Amode, Gpr, OperandVisitor, OperandVisitorImpl, SyntheticAmode, WritableGpr};
use arbitrary::Arbitrary;
use cranelift_assembler as asm;

/// Define the types of registers Cranelift will use.
#[derive(Clone, Debug)]
pub struct CraneliftRegisters;
impl asm::Registers for CraneliftRegisters {
    type ReadGpr = Gpr;
    type ReadWriteGpr = PairedGpr;
}

/// A pair of registers, one for reading and one for writing.
///
/// Due to how Cranelift's SSA form, we must track the read and write registers
/// separately prior to register allocation. Once register allocation is
/// complete, we expect the hardware encoding for both `read` and `write` to be
/// the same.
#[derive(Clone, Copy, Debug)]
pub struct PairedGpr {
    pub(crate) read: Gpr,
    pub(crate) write: WritableGpr,
}

impl asm::AsReg for PairedGpr {
    fn enc(&self) -> u8 {
        let PairedGpr { read, write } = self;
        let read = enc(read);
        let write = enc(&write.to_reg());
        assert_eq!(read, write);
        write
    }

    fn new(_: u8) -> Self {
        panic!("disallow creation of new assembler registers")
    }
}

impl<'a> Arbitrary<'a> for PairedGpr {
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        unimplemented!("assembler fuzzing is not implemented at this level")
    }
}

/// This bridges the gap between codegen and assembler register types.
impl asm::AsReg for Gpr {
    fn enc(&self) -> u8 {
        enc(self)
    }

    fn new(_: u8) -> Self {
        panic!("disallow creation of new assembler registers")
    }
}

impl<'a> Arbitrary<'a> for Gpr {
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        unimplemented!("assembler fuzzing is not implemented at this level")
    }
}

/// A helper method for extracting the hardware encoding of a register.
#[inline]
fn enc(gpr: &Gpr) -> u8 {
    if let Some(real) = gpr.to_reg().to_real_reg() {
        real.hw_enc()
    } else {
        unreachable!()
    }
}

/// A wrapper to implement the `cranelift-assembler` register allocation trait,
/// `RegallocVisitor`, in terms of the trait used in Cranelift,
/// `OperandVisitor`.
pub(crate) struct RegallocVisitor<'a, T>
where
    T: OperandVisitorImpl,
{
    pub collector: &'a mut T,
}

impl<'a, T: OperandVisitor> asm::RegisterVisitor<CraneliftRegisters> for RegallocVisitor<'a, T> {
    fn read(&mut self, reg: &mut Gpr) {
        self.collector.reg_use(reg);
    }

    fn read_write(&mut self, reg: &mut PairedGpr) {
        let PairedGpr { read, write } = reg;
        self.collector.reg_use(read);
        self.collector.reg_reuse_def(write, 0);
    }

    fn fixed_read(&mut self, _reg: &Gpr) {
        todo!()
    }

    fn fixed_read_write(&mut self, _reg: &PairedGpr) {
        todo!()
    }
}

impl Into<asm::Amode<Gpr>> for SyntheticAmode {
    fn into(self) -> asm::Amode<Gpr> {
        match self {
            SyntheticAmode::Real(amode) => match amode {
                Amode::ImmReg {
                    simm32,
                    base,
                    flags,
                } => asm::Amode::ImmReg {
                    simm32: asm::Simm32PlusKnownOffset {
                        simm32: simm32.into(),
                        offset: None,
                    },
                    base: Gpr::unwrap_new(base),
                    trap: flags.trap_code().map(Into::into),
                },
                Amode::ImmRegRegShift {
                    simm32,
                    base,
                    index,
                    shift,
                    flags,
                } => asm::Amode::ImmRegRegShift {
                    base,
                    index: asm::MinusRsp::new(index),
                    scale: asm::Scale::new(shift),
                    simm32: simm32.into(),
                    trap: flags.trap_code().map(Into::into),
                },
                Amode::RipRelative { target } => asm::Amode::RipRelative {
                    target: asm::DeferredTarget::Label(asm::Label(target.as_u32())),
                },
            },
            SyntheticAmode::IncomingArg { offset } => asm::Amode::ImmReg {
                base: Gpr::unwrap_new(regs::rbp()),
                simm32: asm::Simm32PlusKnownOffset {
                    simm32: (-i32::try_from(offset).unwrap()).into(),
                    offset: Some(offsets::KEY_INCOMING_ARG),
                },
                trap: None,
            },
            SyntheticAmode::SlotOffset { simm32 } => asm::Amode::ImmReg {
                base: Gpr::unwrap_new(regs::rbp()),
                simm32: asm::Simm32PlusKnownOffset {
                    simm32: simm32.into(),
                    offset: Some(offsets::KEY_SLOT_OFFSET),
                },
                trap: None,
            },
            SyntheticAmode::ConstantOffset(vcode_constant) => asm::Amode::RipRelative {
                target: asm::DeferredTarget::Constant(asm::Constant(vcode_constant.as_u32())),
            },
        }
    }
}

/// Keep track of the offset slots to fill in during emission; see
/// `KnownOffsetTable`.
pub mod offsets {
    pub const KEY_INCOMING_ARG: usize = 0;
    pub const KEY_SLOT_OFFSET: usize = 1;
}

impl Into<asm::TrapCode> for TrapCode {
    fn into(self) -> asm::TrapCode {
        asm::TrapCode(self.as_raw())
    }
}
