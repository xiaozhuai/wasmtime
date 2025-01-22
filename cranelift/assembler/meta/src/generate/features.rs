use super::{fmtln, Formatter};
use crate::{dsl, generate::generate_derive};

impl dsl::Features {
    /// This generator will emit two separate things:
    /// - a list of (`var`, `expr`) tuples to materialize the terms
    /// - an expression string containing the boolean logic matching the feature tree
    pub fn generate(&self, terms: &mut Vec<(String, dsl::Flag)>, parens: bool) -> String {
        use dsl::Features::*;
        let mut expr = vec![];
        match self {
            None => {
                expr.push("true".to_string());
            }
            Flag(flag) => {
                let t = format!("f{}", terms.len());
                terms.push((t.clone(), *flag));
                expr.push(t);
            }
            And(lhs, rhs) => {
                let lhs = lhs.generate(terms, true);
                let rhs = rhs.generate(terms, true);
                expr.push(if parens {
                    format!("({lhs} && {rhs})")
                } else {
                    format!("{lhs} && {rhs}")
                });
            }
            Or(lhs, rhs) => {
                let lhs = lhs.generate(terms, true);
                let rhs = rhs.generate(terms, true);
                expr.push(if parens {
                    format!("({lhs} || {rhs})")
                } else {
                    format!("{lhs} || {rhs}")
                });
            }
        }
        expr.join(" ")
    }
}

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
