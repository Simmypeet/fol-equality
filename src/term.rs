use std::fmt::Debug;
use std::hash::Hash;

/// Represents a term in a function-symbol.
///
/// This represents something like `f(x, g(y))`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Function<Literal: Ord + Eq + Hash + Clone + Debug> {
    /// The name of the function.
    pub symbol: Literal,

    /// The arguments supplied to the function.
    pub arguments: Vec<Term<Literal>>,
}

/// Represents a term which can be normalized into another term without mapping equalities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Normalizable<Literal: Ord + Eq + Hash + Clone + Debug> {
    /// The literal identifier.
    pub symbol: Literal,

    /// The arguments supplied to the function.
    pub arguments: Vec<Term<Literal>>,
}

/// Represents a term used in equalities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Term<Literal: Ord + Eq + Hash + Clone + Debug> {
    Literal(Literal),
    Function(Function<Literal>),
    Normalizable(Normalizable<Literal>),
}
