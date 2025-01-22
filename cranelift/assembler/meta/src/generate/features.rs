use super::{fmtln, Formatter};
use crate::{dsl, generate::generate_derive};

impl dsl::Flag {
    pub fn name(&self) -> &str {
        use dsl::Flag::*;
        match self {
            _64b => "_64b",
            compat => "compat",
        }
    }

    pub fn generate_enum(f: &mut Formatter) {
        use dsl::Flag::*;

        // N.B.: it is critical that this list contains _all_ variants of the `Flag` enumeration
        // here at the `meta` level so that we can accurately transcribe them to a structure
        // available in the generated layer above. If this list is incomplete, we will see compile
        // errors for generated functions that use the missing variants.
        const ALL: &[dsl::Flag] = &[_64b, compat];

        generate_derive(f);
        fmtln!(f, "pub enum Flag {{");
        f.indent(|f| {
            for flag in ALL {
                let name = flag.name();
                let pos = *flag as usize;
                fmtln!(f, "{name} = {pos},");
            }
        });
        fmtln!(f, "}}");
    }
}
