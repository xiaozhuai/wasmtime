use core::fmt;
use std::ops::BitOr;

#[derive(PartialEq)]
pub struct Features {
    pub flags: Vec<Flag>,
}

impl fmt::Display for Features {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.flags.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(" | "))
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Flag {
    _64b = 0,
    compat = 1,
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Flag::_64b => write!(f, "64-bit"),
            Flag::compat => write!(f, "compat"),
        }
    }
}

impl From<Flag> for Features {
    fn from(flag: Flag) -> Self {
        Features { flags: vec![flag] }
    }
}

impl From<Option<Flag>> for Features {
    fn from(flag: Option<Flag>) -> Self {
        Features { flags: flag.into_iter().collect() }
    }
}

impl BitOr for Flag {
    type Output = Features;
    fn bitor(self, rhs: Self) -> Self::Output {
        Features { flags: vec![self.into(), rhs.into()] }
    }
}
