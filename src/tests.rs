use std::fmt::Debug;

use proptest::{
    arbitrary::Arbitrary,
    prop_assert, prop_oneof, proptest,
    strategy::{BoxedStrategy, Strategy},
    test_runner::TestCaseError,
};

use crate::{equals, visitor::Visitor, Function, Premise, Term};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ID(usize);

impl Arbitrary for ID {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        (0..=1000usize).prop_map(ID).boxed()
    }
}

impl Arbitrary for Function<ID> {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = Option<BoxedStrategy<Term<ID>>>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        let args = args.unwrap_or_else(Term::arbitrary);

        (ID::arbitrary(), proptest::collection::vec(args, 0..=4))
            .prop_map(|(symbol, arguments)| Self { symbol, arguments })
            .boxed()
    }
}

impl Arbitrary for Term<ID> {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        ID::arbitrary()
            .prop_map(Term::Literal)
            .prop_recursive(4, 16, 4, |inner| {
                prop_oneof![
                    2 => Function::arbitrary_with(Some(inner.clone())).prop_map(Term::Function),
                    1 => inner
                ]
            })
            .boxed()
    }
}

/// A proprty of an equality system.
pub trait Property: 'static + Send + Sync + Debug {
    /// Determines wether the terms generated by this property require a premise to be true
    fn requires_premise(&self) -> bool;

    /// Generates a term which will be tested in the property.
    fn terms(&self) -> (Term<ID>, Term<ID>);

    /// Applies the property to the premise.
    fn apply(&self, premise: &mut Premise<ID>) -> bool;
}

impl Arbitrary for Box<dyn Property> {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        let leaf = Identity::arbitrary().prop_map(|x| Box::new(x) as _);

        leaf.prop_recursive(64, 128, 2, |inner| {
            prop_oneof![
                Mapping::arbitrary_with(Some(inner.clone())).prop_map(|x| Box::new(x) as _),
                Unification::arbitrary_with(Some(inner.clone())).prop_map(|x| Box::new(x) as _),
                Normalization::arbitrary_with(Some(inner.clone())).prop_map(|x| Box::new(x) as _),
            ]
        })
        .boxed()
    }
}

/// A property which generates two identical terms for testing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    term: Term<ID>,
}

impl Arbitrary for Identity {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        Term::arbitrary().prop_map(|term| Self { term }).boxed()
    }
}

impl Property for Identity {
    fn requires_premise(&self) -> bool {
        false
    }

    fn terms(&self) -> (Term<ID>, Term<ID>) {
        (self.term.clone(), self.term.clone())
    }

    fn apply(&self, _: &mut Premise<ID>) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Mapping {
    lhs_property: Box<dyn Property>,
    rhs_property: Box<dyn Property>,
}

impl Arbitrary for Mapping {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = Option<BoxedStrategy<Box<dyn Property>>>;

    fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
        let strat = arg.unwrap_or_else(Box::<dyn Property>::arbitrary);

        (strat.clone(), strat)
            .prop_map(|(lhs_property, rhs_property)| Self {
                lhs_property,
                rhs_property,
            })
            .prop_filter("filter out trivially equals case", |prop| {
                let (l_lhs, _) = prop.lhs_property.terms();
                let (_, r_rhs) = prop.rhs_property.terms();

                l_lhs != r_rhs
            })
            .boxed()
    }
}

impl Property for Mapping {
    fn requires_premise(&self) -> bool {
        true
    }

    fn terms(&self) -> (Term<ID>, Term<ID>) {
        let (l_lhs, _) = self.lhs_property.terms();
        let (_, r_rhs) = self.rhs_property.terms();

        (l_lhs, r_rhs)
    }

    fn apply(&self, premise: &mut Premise<ID>) -> bool {
        let (_, l_rhs) = self.lhs_property.terms();
        let (r_lhs, _) = self.rhs_property.terms();

        premise.insert(l_rhs, r_lhs);

        self.lhs_property.apply(premise) && self.rhs_property.apply(premise)
    }
}

#[derive(Debug)]
pub struct Unification {
    arguments_property: Vec<Box<dyn Property>>,
    symbol: ID,
}

impl Arbitrary for Unification {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = Option<BoxedStrategy<Box<dyn Property>>>;

    fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
        let strat = arg.unwrap_or_else(Box::<dyn Property>::arbitrary);

        (ID::arbitrary(), proptest::collection::vec(strat, 0..=4))
            .prop_map(|(symbol, arguments_property)| Self {
                arguments_property,
                symbol,
            })
            .boxed()
    }
}

impl Property for Unification {
    fn requires_premise(&self) -> bool {
        self.arguments_property.iter().any(|x| x.requires_premise())
    }

    fn terms(&self) -> (Term<ID>, Term<ID>) {
        let mut lhs_arguments = Vec::new();
        let mut rhs_arguments = Vec::new();

        for property in &self.arguments_property {
            let (lhs, rhs) = property.terms();

            lhs_arguments.push(lhs);
            rhs_arguments.push(rhs);
        }

        (
            Term::Function(Function {
                symbol: self.symbol,
                arguments: lhs_arguments,
            }),
            Term::Function(Function {
                symbol: self.symbol,
                arguments: rhs_arguments,
            }),
        )
    }

    fn apply(&self, premise: &mut Premise<ID>) -> bool {
        for property in &self.arguments_property {
            if !property.apply(premise) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct Normalization {
    property: Box<dyn Property>,
    literal_identifier: ID,
    substituted_term: Term<ID>,
    normalizable_literal: ID,
    normalizable_at_lhs: bool,
}

struct TermCollector {
    terms: Vec<Term<ID>>,
}

impl Visitor<ID> for TermCollector {
    fn visit(&mut self, term: &Term<ID>) -> bool {
        self.terms.push(term.clone());
        true
    }
}

impl Arbitrary for Normalization {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = Option<BoxedStrategy<Box<dyn Property>>>;

    fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
        let strat = arg.unwrap_or_else(Box::<dyn Property>::arbitrary);

        (
            proptest::num::usize::ANY,
            proptest::bool::ANY,
            strat,
            ID::arbitrary(),
            ID::arbitrary(),
        )
            .prop_filter_map(
                "filter out crashin ids",
                |(modulo, normalizable_at_lhs, strat, normalizable_literal, literal_identifier)| {
                    let equiv = if normalizable_at_lhs {
                        strat.terms().0
                    } else {
                        strat.terms().1
                    };

                    let mut term_collector = TermCollector { terms: Vec::new() };
                    equiv.visit(&mut term_collector);

                    if term_collector
                        .terms
                        .contains(&Term::Literal(normalizable_literal))
                    {
                        return None;
                    }

                    let term_index = modulo % term_collector.terms.len();
                    let substituted_term = term_collector.terms.remove(term_index);

                    Some(Self {
                        property: strat,
                        literal_identifier,
                        substituted_term,
                        normalizable_literal,
                        normalizable_at_lhs,
                    })
                },
            )
            .prop_filter("filter out trivially equals", |x| {
                let (lhs, rhs) = x.terms();

                lhs != rhs
            })
            .boxed()
    }
}

impl Property for Normalization {
    fn requires_premise(&self) -> bool {
        true
    }

    fn terms(&self) -> (Term<ID>, Term<ID>) {
        let normalizable = Term::Normalizable(crate::Normalizable {
            symbol: self.literal_identifier,
            arguments: vec![self.substituted_term.clone()],
        });

        if self.normalizable_at_lhs {
            (normalizable, self.property.terms().1)
        } else {
            (self.property.terms().0, normalizable)
        }
    }

    fn apply(&self, premise: &mut Premise<ID>) -> bool {
        if !self.property.apply(premise) {
            return false;
        }

        premise.insert_normalization(self.literal_identifier, vec![self.normalizable_literal], {
            let (lhs, rhs) = self.property.terms();

            let mut normalized = if self.normalizable_at_lhs { lhs } else { rhs };

            normalized.apply(
                &self.substituted_term,
                &Term::Literal(self.normalizable_literal),
            );

            normalized
        })
    }
}

proptest! {
    #[test]
    fn property_based_testing(
        property in Box::<dyn Property>::arbitrary()
    ) {
        let (term1, term2) = property.terms();
        let mut premise = Premise::<ID>::default();

        if property.requires_premise() {
            // without premise the equality should not hold
            prop_assert!(!equals(&term1, &term2, &premise));

            if !property.apply(&mut premise) {
                return Err(TestCaseError::reject("skip failed property application"))
            }
        }

        // now the equality should hold
        prop_assert!(equals(&term1, &term2, &premise));
        prop_assert!(equals(&term2, &term1, &premise));
    }
}

#[test]
fn reflixivity() {
    let term = Term::Literal(ID(0));
    let premise = Premise::<ID>::default();

    assert!(equals(&term, &term, &premise));
}

#[test]
fn symmetry() {
    let term1 = Term::Literal(ID(0));
    let term2 = Term::Literal(ID(1));

    let not_equal = Term::Literal(ID(2));
    let mut premise = Premise::<ID>::default();

    premise.insert(term1.clone(), term2.clone());

    assert!(equals(&term1, &term2, &premise));
    assert!(equals(&term2, &term1, &premise));

    assert!(!equals(&term1, &not_equal, &premise));
    assert!(!equals(&term2, &not_equal, &premise));

    assert!(!equals(&not_equal, &term1, &premise));
    assert!(!equals(&not_equal, &term2, &premise));
}

#[test]
fn transitivity() {
    let term1 = Term::Literal(ID(0));
    let term2 = Term::Literal(ID(1));
    let term3 = Term::Literal(ID(2));

    let not_equal = Term::Literal(ID(3));

    let mut premise = Premise::<ID>::default();

    premise.insert(term1.clone(), term2.clone());
    premise.insert(term2.clone(), term3.clone());

    assert!(equals(&term1, &term2, &premise));
    assert!(equals(&term2, &term3, &premise));
    assert!(equals(&term1, &term3, &premise));

    assert!(!equals(&term1, &not_equal, &premise));
    assert!(!equals(&term2, &not_equal, &premise));
    assert!(!equals(&term3, &not_equal, &premise));

    assert!(!equals(&not_equal, &term1, &premise));
    assert!(!equals(&not_equal, &term2, &premise));
    assert!(!equals(&not_equal, &term3, &premise));
}

#[test]
fn congruence() {
    let term1 = Term::Function(Function {
        symbol: ID(0),
        arguments: vec![Term::Literal(ID(1)), Term::Literal(ID(2))],
    });
    let term2 = Term::Function(Function {
        symbol: ID(0),
        arguments: vec![Term::Literal(ID(3)), Term::Literal(ID(4))],
    });
    let not_equal = Term::Function(Function {
        symbol: ID(0),
        arguments: vec![Term::Literal(ID(5)), Term::Literal(ID(6))],
    });

    let mut premise = Premise::<ID>::default();

    premise.insert(Term::Literal(ID(1)), Term::Literal(ID(3)));
    premise.insert(Term::Literal(ID(2)), Term::Literal(ID(4)));

    assert!(equals(&term1, &term2, &premise));
    assert!(equals(&term2, &term1, &premise));

    assert!(!equals(&term1, &not_equal, &premise));
    assert!(!equals(&term2, &not_equal, &premise));

    assert!(!equals(&not_equal, &term1, &premise));
    assert!(!equals(&not_equal, &term2, &premise));
}

#[test]
fn recursive_term() {
    let premise = Premise::new_with_equalities([(
        Term::Literal(ID(0)),
        Term::Function(Function {
            symbol: ID(0),
            arguments: vec![Term::Literal(ID(0))],
        }),
    )]);

    let lhs = Term::Function(Function {
        symbol: ID(0),
        arguments: vec![Term::Literal(ID(0))],
    });
    let rhs = Term::Function(Function {
        symbol: ID(0),
        arguments: vec![Term::Function(Function {
            symbol: ID(0),
            arguments: vec![Term::Function(Function {
                symbol: ID(0),
                arguments: vec![Term::Function(Function {
                    symbol: ID(0),
                    arguments: vec![Term::Literal(ID(0))],
                })],
            })],
        })],
    });

    assert!(equals(&lhs, &rhs, &premise));
    assert!(equals(&rhs, &lhs, &premise));
}
