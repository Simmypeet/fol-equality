# FOL Equality

An implementation of equality checking in first-order logic. This is not a complete
first-order theorem prover, but rather a tool for checking equality of terms.

These are four axioms of equality this implementation follows:

- $\forall x (x = x)$ (reflexitivity)
- $\forall x \forall y (x = y \iff y = x)$ (symmetry)
- $\forall x \forall y \forall z (x = y \land y = z \implies x = z)$ (transitivity)
- $\forall x_1 \ldots \forall x_n \forall y_1 \ldots \forall y_n (x_1 = y_1 \land \ldots \land x_n = y_n \implies f(x_1, \ldots, x_n) = f(y_1, \ldots, y_n))$ (congruence)
  
## Example

```rust
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

```
