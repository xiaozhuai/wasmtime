//! Register allocation.

pub trait OperandVisitor<R> {
    fn read(&mut self, reg: &mut R);
    fn read_write(&mut self, reg: &mut R);
    fn fixed_read(&mut self, reg: &R);
    fn fixed_read_write(&mut self, reg: &R);
}
