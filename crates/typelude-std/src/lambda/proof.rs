//! Type-Level Theorem Proving
//!
//! Provides type-level witnesses for proofs like equality and commutativity.

use std::marker::PhantomData;

use typelude_std::core::Eval;
/// Reflexivity witness: `Refl<A>` proves that `A` equals itself.
pub struct LRefl<A>(PhantomData<A>);

impl<A> Eval for LRefl<A> {
    type Output = LRefl<A>;
}
/// Marker trait asserting that two types are equal.
/// If `A: TypeEq<B>`, then `A` and `B` are the same type.
pub trait LTypeEq<B> {
    type Proof;
}

/// Any type is equal to itself (reflexivity).
impl<A> LTypeEq<A> for A {
    type Proof = LRefl<A>;
}
/// Symmetry witness: transforms `Proof<A, B>` into `Proof<B, A>`.
pub struct LSym<Proof>(PhantomData<Proof>);

impl<A> Eval for LSym<LRefl<A>> {
    type Output = LRefl<A>;
}
/// Transitivity witness: combines two proofs.
pub struct LTrans<Proof1, Proof2>(PhantomData<(Proof1, Proof2)>);

impl<A> Eval for LTrans<LRefl<A>, LRefl<A>> {
    type Output = LRefl<A>;
}
/// Congruence witness: lifts equality through a type constructor.
pub struct LCong<F, Proof>(PhantomData<(F, Proof)>);

// We can't directly express this in Rust without HKT, but we can provide
// specific instances.

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::Evaluate;

    use super::*;

    struct A;

    #[test]
    fn test_reflexivity() {
        // A is equal to A
        type Proof = <A as LTypeEq<A>>::Proof;
        assert_type_eq_all!(Proof, LRefl<A>);
    }

    #[test]
    fn test_symmetry() {
        // Sym<Refl<A>> = Refl<A>
        type Proof = Evaluate<LSym<LRefl<A>>>;
        assert_type_eq_all!(Proof, LRefl<A>);
    }

    #[test]
    fn test_transitivity() {
        // Trans<Refl<A>, Refl<A>> = Refl<A>
        type Proof = Evaluate<LTrans<LRefl<A>, LRefl<A>>>;
        assert_type_eq_all!(Proof, LRefl<A>);
    }
}
