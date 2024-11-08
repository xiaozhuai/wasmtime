//! Register allocation.

pub trait RegallocVisitor {
    fn read(&mut self, reg: u8);
    fn read_write(&mut self, reg: u8);
    fn fixed_read(&mut self, reg: u8);
    fn fixed_read_write(&mut self, reg: u8);
}
