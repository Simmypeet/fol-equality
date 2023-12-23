use std::fmt::Debug;
use std::hash::Hash;

use crate::{Function, Normalizable, Term};

impl<Literal: Ord + Eq + Hash + Clone + Debug> Term<Literal> {
    /// Applies a substitution to the term.
    pub fn apply(&mut self, from: &Self, to: &Self) {
        if self == from {
            *self = to.clone();
        }

        match self {
            Self::Literal(_) => {}
            Self::Function(Function { arguments, .. })
            | Self::Normalizable(Normalizable { arguments, .. }) => {
                for argument in arguments {
                    argument.apply(from, to);
                }
            }
        }
    }
}
