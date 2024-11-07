//! Contains the encoding machinery for the various x64 instruction formats.
use super::inst::LabelUse;
use crate::{ir::TrapCode, isa::x64, machinst::MachBuffer, MachLabel};
use std::vec::Vec;
pub mod evex;
pub mod rex;
pub mod vex;

/// The encoding formats in this module all require a way of placing bytes into
/// a buffer.
pub trait ByteSink {
    /// Add 1 byte to the code section.
    fn put1(&mut self, _: u8);

    /// Add 2 bytes to the code section.
    fn put2(&mut self, _: u16);

    /// Add 4 bytes to the code section.
    fn put4(&mut self, _: u32);

    /// Add 8 bytes to the code section.
    fn put8(&mut self, _: u64);

    /// TODO: I would prefer to keep this away from this trait (but see how
    /// `Amode::RipRelative` is lowered).
    fn cur_offset(&self) -> u32;

    /// TODO: I would prefer to keep this away from this trait (but see how
    /// `Amode::RipRelative` is lowered).
    fn use_label_at_offset(&mut self, offset: u32, label: MachLabel, kind: LabelUse);

    /// TODO: I would prefer to keep this away from this trait (but see
    /// how we lower `Amode`).
    fn add_trap(&mut self, code: TrapCode);
}

impl ByteSink for MachBuffer<x64::inst::Inst> {
    fn put1(&mut self, value: u8) {
        self.put1(value)
    }

    fn put2(&mut self, value: u16) {
        self.put2(value)
    }

    fn put4(&mut self, value: u32) {
        self.put4(value)
    }

    fn put8(&mut self, value: u64) {
        self.put8(value)
    }

    fn cur_offset(&self) -> u32 {
        self.cur_offset()
    }

    fn use_label_at_offset(&mut self, offset: u32, label: MachLabel, kind: LabelUse) {
        self.use_label_at_offset(offset, label, kind);
    }

    fn add_trap(&mut self, code: TrapCode) {
        self.add_trap(code);
    }
}

/// Provide a convenient implementation for testing.
impl ByteSink for Vec<u8> {
    fn put1(&mut self, v: u8) {
        self.extend_from_slice(&[v])
    }

    fn put2(&mut self, v: u16) {
        self.extend_from_slice(&v.to_le_bytes())
    }

    fn put4(&mut self, v: u32) {
        self.extend_from_slice(&v.to_le_bytes())
    }

    fn put8(&mut self, v: u64) {
        self.extend_from_slice(&v.to_le_bytes())
    }

    fn cur_offset(&self) -> u32 {
        self.len().try_into().unwrap()
    }

    fn use_label_at_offset(&mut self, _: u32, _: MachLabel, _: LabelUse) {}

    fn add_trap(&mut self, _: TrapCode) {}
}
