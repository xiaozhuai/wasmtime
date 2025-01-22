use crate::Flag;

pub struct AvailableFeatures(u32);

impl AvailableFeatures {
    pub fn new(flags: impl Iterator<Item = Flag>) -> Self {
        let mut avail = 0;
        for f in flags {
            let pos = f as usize;
            avail |= 1 << pos;
        }
        Self(avail)
    }

    pub fn index(&self, index: Flag) -> bool {
        self.0 & (1 << index as usize) == 1
    }
}
