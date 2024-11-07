use core::fmt;

#[derive(PartialEq)]
pub enum Features {
    None,
    Flag(Flag),
    And(Box<Features>, Box<Features>),
    Or(Box<Features>, Box<Features>),
}

impl fmt::Display for Features {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Features::{And, Flag, None, Or};
        match self {
            None => write!(f, ""),
            Flag(flag) => write!(f, "{flag}"),
            And(left, right) => write!(f, "{left} && {right}"),
            Or(left, right) => write!(f, "{left} || {right}"),
        }
    }
}

#[derive(PartialEq)]
pub enum Flag {}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
