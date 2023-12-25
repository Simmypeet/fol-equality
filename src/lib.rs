//! Implementation of the equality algorithm in the First-Order Logic system.

mod premise;
mod substitution;
mod term;
mod visitor;

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

pub use premise::Normalization;
pub use premise::Premise;

pub use term::Function;
pub use term::Normalizable;
pub use term::Term;

fn equals_by_unification<Literal: Ord + Eq + Hash + Clone + Debug>(
    term1: &Term<Literal>,
    term2: &Term<Literal>,
    premise: &Premise<Literal>,
    visited: &mut HashSet<(Term<Literal>, Term<Literal>)>,
) -> bool {
    match (term1, term2) {
        (
            Term::Function(Function {
                symbol: name1,
                arguments: args1,
            }),
            Term::Function(Function {
                symbol: name2,
                arguments: args2,
            }),
        )
        | (
            Term::Normalizable(Normalizable {
                symbol: name1,
                arguments: args1,
            }),
            Term::Normalizable(Normalizable {
                symbol: name2,
                arguments: args2,
            }),
        ) if name1 == name2 && args1.len() == args2.len() => {
            let mut unification_succeed = true;
            for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                if !dfs(arg1, arg2, premise, visited) {
                    unification_succeed = false;
                    break;
                }
            }

            unification_succeed
        }
        _ => false,
    }
}

fn equals_by_normalization<Literal: Ord + Eq + Hash + Clone + Debug>(
    term1: &Term<Literal>,
    term2: &Term<Literal>,
    premise: &Premise<Literal>,
    visited: &mut HashSet<(Term<Literal>, Term<Literal>)>,
) -> bool {
    if let Term::Normalizable(term1) = term1 {
        if let Some(normalization) = premise.get_normalization(&term1.symbol) {
            if let Some(equivalence) = normalization.equivalence(&term1.arguments) {
                return dfs(&equivalence, term2, premise, visited);
            }
        }
    }

    if let Term::Normalizable(term2) = term2 {
        if let Some(normalization) = premise.get_normalization(&term2.symbol) {
            if let Some(equivalence) = normalization.equivalence(&term2.arguments) {
                return dfs(term1, &equivalence, premise, visited);
            }
        }
    }

    false
}

fn dfs<Literal: Eq + Ord + Hash + Clone + Debug>(
    term: &Term<Literal>,
    term2: &Term<Literal>,
    premise: &Premise<Literal>,
    visited: &mut HashSet<(Term<Literal>, Term<Literal>)>,
) -> bool {
    if term == term2 {
        return true;
    }

    if !visited.insert((term.clone(), term2.clone())) {
        // already visited
        return false;
    }

    // try to unify
    if equals_by_unification(term, term2, premise, visited) {
        visited.remove(&(term.clone(), term2.clone()));
        return true;
    }

    // try to normalize
    if equals_by_normalization(term, term2, premise, visited) {
        visited.remove(&(term.clone(), term2.clone()));
        return true;
    }

    // try to look for a mapping in the premise
    if let Some(equivalences) = premise.equalities().get(term) {
        for equivalence in equivalences {
            if dfs(equivalence, term2, premise, visited) {
                visited.remove(&(term.clone(), term2.clone()));
                return true;
            }
        }
    }
    if let Some(equivalences) = premise.equalities().get(term2) {
        for equivalence in equivalences {
            if dfs(term, equivalence, premise, visited) {
                visited.remove(&(term.clone(), term2.clone()));
                return true;
            }
        }
    }

    // try to unify/normalize the premise
    for (key, values) in premise.equalities() {
        if equals_by_unification(term, key, premise, visited) {
            for value in values {
                if dfs(value, term2, premise, visited) {
                    visited.remove(&(term.clone(), term2.clone()));
                    return true;
                }
            }
        }

        if equals_by_unification(key, term2, premise, visited) {
            for value in values {
                if dfs(term, value, premise, visited) {
                    visited.remove(&(term.clone(), term2.clone()));
                    return true;
                }
            }
        }

        if equals_by_normalization(term, key, premise, visited) {
            for value in values {
                if dfs(value, term2, premise, visited) {
                    visited.remove(&(term.clone(), term2.clone()));
                    return true;
                }
            }
        }

        if equals_by_normalization(key, term2, premise, visited) {
            for value in values {
                if dfs(term, value, premise, visited) {
                    visited.remove(&(term.clone(), term2.clone()));
                    return true;
                }
            }
        }
    }

    false
}

/// Determines if two terms are equal.
#[must_use]
pub fn equals<Literal: Ord + Eq + Hash + Clone + Debug>(
    term1: &Term<Literal>,
    term2: &Term<Literal>,
    premise: &Premise<Literal>,
) -> bool {
    // guaranteed to have at least 32K of stack
    let mut visited = HashSet::new();

    dfs(term1, term2, premise, &mut visited)
}

#[cfg(test)]
mod tests;
