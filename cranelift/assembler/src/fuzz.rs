//! Fuzzing utilities.

use crate::Inst;
use capstone::arch::{BuildsCapstone, BuildsCapstoneSyntax};
use cranelift_codegen::{control::ControlPlane, MachBuffer, VCodeConstants};

/// Generate a random assembly instruction and check its encoding and
/// pretty-printing against a known-good disassembler.
pub fn roundtrip(inst: &Inst) {
    // Check that we can actually assemble this instruction.
    let assembled = assemble(inst);
    let expected = disassemble(&assembled);

    // Check that our pretty-printed output matches the known-good output.
    let expected = expected.split_once(' ').unwrap().1;
    let actual = inst.to_string();
    if expected != actual {
        println!("> {inst}");
        println!("  debug: {inst:x?}");
        println!("  assembled: {}", pretty_print_hexadecimal(&assembled));
        assert_eq!(expected, &actual);
    }
}

/// See `cranelift/codegen/src/isa/x64/inst/emit_tests.rs`; this is suboptimal,
/// though (TODO).
fn assemble(insn: &Inst) -> Vec<u8> {
    let ctrl_plane = &mut ControlPlane::default();
    let constants = VCodeConstants::default();
    let mut buffer = MachBuffer::new();

    insn.encode(&mut buffer);

    // Allow one label just after the instruction (so the offset is 0).
    let label = buffer.get_label();
    buffer.bind_label(label, ctrl_plane);

    let buffer = buffer.finish(&constants, ctrl_plane);
    buffer.data().to_owned()
}

/// Building a new `Capstone` each time is suboptimal (TODO).
fn disassemble(assembled: &[u8]) -> String {
    let cs = capstone::Capstone::new()
        .x86()
        .mode(capstone::arch::x86::ArchMode::Mode64)
        .syntax(capstone::arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("failed to create Capstone object");
    let insns = cs
        .disasm_all(assembled, 0x0)
        .expect("failed to disassemble");
    assert_eq!(insns.len(), 1, "not a single instruction: {assembled:x?}");
    let insn = insns.first().expect("at least one instruction");
    assert_eq!(assembled.len(), insn.len());
    insn.to_string()
}

fn pretty_print_hexadecimal(hex: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(hex.len() * 2);
    for b in hex {
        write!(&mut s, "{b:02X}").unwrap();
    }
    s
}

#[cfg(test)]
mod test {
    use super::*;
    use arbtest::arbtest;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn smoke() {
        let count = AtomicUsize::new(0);
        arbtest(|u| {
            let inst: Inst = u.arbitrary()?;
            roundtrip(&inst);
            println!("#{}: {inst}", count.fetch_add(1, Ordering::SeqCst));
            Ok(())
        })
        .budget_ms(1_000);

        // This will run the `roundtrip` fuzzer for one second. To repeatably
        // test a single input, append `.seed(0x<failing seed>)`.
    }
}
