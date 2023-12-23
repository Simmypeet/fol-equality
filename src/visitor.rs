use crate::{Function, Normalizable, Term};

use std::fmt::Debug;
use std::hash::Hash;

/// A visitor for terms,
pub trait Visitor<Literal: Ord + Eq + Hash + Clone + Debug> {
    /// Visits a term.
    ///
    /// Returns `false` if visiting should be stopped.
    fn visit(&mut self, term: &Term<Literal>) -> bool;
}

impl<Literal: Ord + Eq + Hash + Clone + Debug> Term<Literal> {
    /// Visits the term.
    pub fn visit<V: Visitor<Literal>>(&self, visitor: &mut V) -> bool {
        if !visitor.visit(self) {
            return false;
        }

        match self {
            Self::Literal(_) => true,
            Self::Function(Function { arguments, .. })
            | Self::Normalizable(Normalizable { arguments, .. }) => {
                for argument in arguments {
                    if !argument.visit(visitor) {
                        return false;
                    }
                }

                true
            }
        }
    }
}
