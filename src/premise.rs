use std::fmt::Debug;
use std::hash::Hash;

use std::collections::{BTreeMap, BTreeSet};

use crate::Term;

/// Represents a premise of equalities.
///
/// For example, the premise
///
/// ``` no_run
/// x = y,
/// x = z,
/// z = y,
/// ```
///
/// Then the premise can be represented as
///
/// ```json
/// "Premise": {
///    "equalities": {
///         "x": ["y", "z"],
///         "y": ["x", "z"],
///         "z": ["y"]
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Premise<Literal: Ord + Eq + Hash + Clone + Debug> {
    /// A map from a term to a set of terms that are equal to it.
    equalities: BTreeMap<Term<Literal>, BTreeSet<Term<Literal>>>,
}

impl<Literal: Ord + Eq + Hash + Clone + Debug> Default for Premise<Literal> {
    fn default() -> Self {
        Self {
            equalities: BTreeMap::default(),
        }
    }
}

impl<Literal: Ord + Eq + Hash + Clone + Debug> Premise<Literal> {
    /// Returns the equalities in the premise.
    #[must_use]
    pub const fn equalities(&self) -> &BTreeMap<Term<Literal>, BTreeSet<Term<Literal>>> {
        &self.equalities
    }

    /// Creates a new premise with pre-defined equalities.
    pub fn new_with_equalities(
        terms: impl IntoIterator<Item = (Term<Literal>, Term<Literal>)>,
    ) -> Self {
        let mut premise = Self::default();

        for (term1, term2) in terms {
            premise.insert(term1, term2);
        }

        premise
    }

    /// Inserts a new equality into the premise.
    pub fn insert(&mut self, term1: Term<Literal>, term2: Term<Literal>) {
        self.equalities
            .entry(term1.clone())
            .or_default()
            .insert(term2.clone());
        self.equalities.entry(term2).or_default().insert(term1);
    }
}
