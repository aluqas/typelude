//! Type-Level Theorem Proving
//!
//! Provides type-level witnesses for proofs like equality and commutativity.

use std::marker::PhantomData;

use super::Lambda;

// =========================================================================
// Reflexivity: Proof that A == A
// =========================================================================

/// Reflexivity witness: `Refl<A>` proves that `A` equals itself.
pub struct Refl<A>(PhantomData<A>);

impl<A> Lambda for Refl<A> {
    type Output = Refl<A>;
}

// =========================================================================
// Type Equality
// =========================================================================

/// Marker trait asserting that two types are equal.
/// If `A: TypeEq<B>`, then `A` and `B` are the same type.
pub trait TypeEq<B> {
    type Proof;
}

/// Any type is equal to itself (reflexivity).
impl<A> TypeEq<A> for A {
    type Proof = Refl<A>;
}

// =========================================================================
// Symmetry: If A == B then B == A
// =========================================================================

/// Symmetry witness: transforms `Proof<A, B>` into `Proof<B, A>`.
pub struct Sym<Proof>(PhantomData<Proof>);

impl<A> Lambda for Sym<Refl<A>> {
    type Output = Refl<A>;
}

// =========================================================================
// Transitivity: If A == B and B == C then A == C
// =========================================================================

/// Transitivity witness: combines two proofs.
pub struct Trans<Proof1, Proof2>(PhantomData<(Proof1, Proof2)>);

impl<A> Lambda for Trans<Refl<A>, Refl<A>> {
    type Output = Refl<A>;
}

// =========================================================================
// Congruence: If A == B then F<A> == F<B>
// =========================================================================

/// Congruence witness: lifts equality through a type constructor.
pub struct Cong<F, Proof>(PhantomData<(F, Proof)>);

// We can't directly express this in Rust without HKT, but we can provide
// specific instances.

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    struct A;
    struct B;

    #[test]
    fn test_reflexivity() {
        // A is equal to A
        type Proof = <A as TypeEq<A>>::Proof;
        assert_type_eq_all!(Proof, Refl<A>);
    }

    #[test]
    fn test_symmetry() {
        // Sym<Refl<A>> = Refl<A>
        type Proof = <Sym<Refl<A>> as Lambda>::Output;
        assert_type_eq_all!(Proof, Refl<A>);
    }

    #[test]
    fn test_transitivity() {
        // Trans<Refl<A>, Refl<A>> = Refl<A>
        type Proof = <Trans<Refl<A>, Refl<A>> as Lambda>::Output;
        assert_type_eq_all!(Proof, Refl<A>);
    }
}
