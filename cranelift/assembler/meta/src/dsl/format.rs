/// Abbreviated constructor for an instruction "format."
///
/// These model what the specification calls "instruction operand encodings,"
/// usually defined in a table after an instruction's opcodes.
pub fn fmt(name: impl Into<String>, operands: impl IntoIterator<Item = Operand>) -> Format {
    Format {
        name: name.into(),
        operands: operands.into_iter().collect(),
    }
}

pub struct Format {
    pub name: String,
    pub operands: Vec<Operand>,
}

impl Format {
    pub fn uses_memory(&self) -> Option<Operand> {
        debug_assert!(
            self.operands
                .iter()
                .copied()
                .filter(Operand::uses_memory)
                .count()
                <= 1
        );
        self.operands.iter().copied().find(Operand::uses_memory)
    }

    pub fn operands_by_bits(&self) -> Vec<u8> {
        self.operands.iter().map(Operand::bits).collect()
    }

    pub fn operands_by_kind(&self) -> Vec<OperandKind> {
        self.operands.iter().map(Operand::kind).collect()
    }

    #[must_use]
    pub fn operands_in_memory_order(&self) -> Vec<Operand> {
        if let Some(mem_op) = self.uses_memory() {
            self.operands[1..].iter().copied().chain([mem_op]).collect()
        } else {
            self.operands.clone()
        }
    }
}

impl core::fmt::Display for Format {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let Format { name, operands } = self;
        write!(
            f,
            "{}({})",
            name,
            operands
                .iter()
                .map(|operand| format!("{operand}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum Operand {
    al,
    ax,
    eax,
    rax,

    imm8,
    imm16,
    imm32,

    r8,
    r16,
    r32,
    r64,

    rm8,
    rm16,
    rm32,
    rm64,
}

impl Operand {
    #[must_use]
    pub fn bits(&self) -> u8 {
        use Operand::{
            al, ax, eax, imm16, imm32, imm8, r16, r32, r64, r8, rax, rm16, rm32, rm64, rm8,
        };
        match self {
            al | imm8 | r8 | rm8 => 8,
            ax | imm16 | r16 | rm16 => 16,
            eax | imm32 | r32 | rm32 => 32,
            rax | r64 | rm64 => 64,
        }
    }

    #[must_use]
    pub fn bytes(&self) -> u8 {
        self.bits() / 8
    }

    #[must_use]
    pub fn uses_memory(&self) -> bool {
        use Operand::{
            al, ax, eax, imm16, imm32, imm8, r16, r32, r64, r8, rax, rm16, rm32, rm64, rm8,
        };
        match self {
            al | ax | eax | rax | imm8 | imm16 | imm32 | r8 | r16 | r32 | r64 => false,
            rm8 | rm16 | rm32 | rm64 => true,
        }
    }

    #[must_use]
    pub fn kind(&self) -> OperandKind {
        use Operand::{
            al, ax, eax, imm16, imm32, imm8, r16, r32, r64, r8, rax, rm16, rm32, rm64, rm8,
        };
        match self {
            al | ax | eax | rax => OperandKind::FixedReg(*self),
            imm8 | imm16 | imm32 => OperandKind::Imm(*self),
            r8 | r16 | r32 | r64 => OperandKind::Reg(*self),
            rm8 | rm16 | rm32 | rm64 => OperandKind::RegMem(*self),
        }
    }
}

impl core::fmt::Display for Operand {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Operand::{
            al, ax, eax, imm16, imm32, imm8, r16, r32, r64, r8, rax, rm16, rm32, rm64, rm8,
        };
        match self {
            al => write!(f, "al"),
            ax => write!(f, "ax"),
            eax => write!(f, "eax"),
            rax => write!(f, "rax"),

            imm8 => write!(f, "imm8"),
            imm16 => write!(f, "imm16"),
            imm32 => write!(f, "imm32"),

            r8 => write!(f, "r8"),
            r16 => write!(f, "r16"),
            r32 => write!(f, "r32"),
            r64 => write!(f, "r64"),

            rm8 => write!(f, "rm8"),
            rm16 => write!(f, "rm16"),
            rm32 => write!(f, "rm32"),
            rm64 => write!(f, "rm64"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OperandKind {
    FixedReg(Operand),
    Imm(Operand),
    Reg(Operand),
    RegMem(Operand),
}
