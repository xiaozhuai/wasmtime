use core::fmt;
use std::ops::{BitAnd, BitOr};

#[derive(PartialEq)]
pub enum Features {
    None,
    Flag(Flag),
    And(Box<Features>, Box<Features>),
    Or(Box<Features>, Box<Features>),
}

impl Features {
    pub fn contains_flag(&self) -> bool {
        use Features::{And, Flag, None, Or};
        match self {
            None => false,
            Flag(_) => true,
            And(lhs, rhs) | Or(lhs, rhs) => lhs.contains_flag() || rhs.contains_flag(),
        }
    }
}

impl fmt::Display for Features {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Features::{And, Flag, None, Or};
        match self {
            None => write!(f, ""),
            Flag(flag) => write!(f, "{flag}"),
            And(left, right) => write!(f, "{left} & {right}"),
            Or(left, right) => write!(f, "{left} | {right}"),
        }
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
        Features::Flag(flag)
    }
}

impl BitAnd for Flag {
    type Output = Features;
    fn bitand(self, rhs: Self) -> Self::Output {
        Features::And(Box::new(self.into()), Box::new(rhs.into()))
    }
}

impl BitOr for Flag {
    type Output = Features;
    fn bitor(self, rhs: Self) -> Self::Output {
        Features::Or(Box::new(self.into()), Box::new(rhs.into()))
    }
}
