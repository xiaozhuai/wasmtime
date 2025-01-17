//! Register allocation.

use crate::Registers;

pub trait OperandVisitor<R: Registers> {
    fn read(&mut self, reg: &mut R::ReadGpr);
    fn read_write(&mut self, reg: &mut R::ReadWriteGpr);
    fn fixed_read(&mut self, reg: &R::ReadGpr);
    fn fixed_read_write(&mut self, reg: &R::ReadWriteGpr);
}
