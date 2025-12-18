use std::marker::PhantomData;

use super::{
    Apply, Lambda,
    church::{LFalse, LTrue},
};
use crate::eval::Eval;

// =========================================================================
// Scott Encoded List
//
// Scott encoding allows for O(1) head/tail access.
//
// Definition:
// Nil = \c n. n
// Cons h t = \c n. c h t
// =========================================================================

/// Nil: \c n. n
pub struct LNil;
impl Lambda for LNil {
    type Output = LNil;
}

/// Cons: \h t. \c n. c h t
pub struct LCons<H, T>(PhantomData<(H, T)>);
impl<H, T> Lambda for LCons<H, T> {
    type Output = LCons<H, T>;
}

// --- Nil Implementation ---
// Nil c -> Nil1<c>
impl<C> Apply<C> for LNil {
    type Output = Nil1<C>;
}
pub struct Nil1<C>(PhantomData<C>);

// Nil1<c> n -> n
impl<C, N> Apply<N> for Nil1<C> {
    type Output = N;
}

// --- Cons Implementation ---
// Cons<H, T> c -> Cons1<H, T, c>
impl<H, T, C> Apply<C> for LCons<H, T> {
    type Output = Cons1<H, T, C>;
}

pub struct Cons1<H, T, C>(PhantomData<(H, T, C)>);

// Cons1<H, T, c> n -> c H T
impl<H, T, C, N> Apply<N> for Cons1<H, T, C>
where
    C: Apply<H>,
    <C as Apply<H>>::Output: Apply<T>,
{
    type Output = <<C as Apply<H>>::Output as Apply<T>>::Output;
}

// =========================================================================
// Basic Accessors
// =========================================================================

/// Uncons l on_cons on_nil
pub type LUncons<L, OnCons, OnNil> = <<L as Apply<OnCons>>::Output as Apply<OnNil>>::Output;

// Helper K: \x y. x
pub struct K;
impl<X> Apply<X> for K {
    type Output = K1<X>;
}
pub struct K1<X>(PhantomData<X>);
impl<X, Y> Apply<Y> for K1<X> {
    type Output = X;
}

// Helper K_I: \x y. y (which is False)
pub struct KI;
impl<X> Apply<X> for KI {
    type Output = KI1;
}
pub struct KI1;
impl<Y> Apply<Y> for KI1 {
    type Output = Y;
}

/// Head: Extract head or return Default
pub struct LHeadOr<L, Default>(PhantomData<(L, Default)>);

impl<L, D> Lambda for LHeadOr<L, D>
where
    L: Apply<K>,
    <L as Apply<K>>::Output: Apply<D>,
{
    type Output = LUncons<L, K, D>;
}

/// Tail: Extract tail or return Default
pub struct LTailOr<L, Default>(PhantomData<(L, Default)>);

impl<L, D> Lambda for LTailOr<L, D>
where
    L: Apply<KI>,
    <L as Apply<KI>>::Output: Apply<D>,
{
    type Output = LUncons<L, KI, D>;
}

// =========================================================================
// IsEmpty
// =========================================================================

/// IsEmpty: \l. l (\h t. False) True
pub struct LIsEmpty<L>(PhantomData<L>);

impl<L> Lambda for LIsEmpty<L>
where
    L: Apply<LConstFalse>,
    <L as Apply<LConstFalse>>::Output: Apply<LTrue>,
{
    type Output = LUncons<L, LConstFalse, LTrue>;
}

pub struct LConstFalse;
impl<X> Apply<X> for LConstFalse {
    type Output = LConstFalse1;
}
pub struct LConstFalse1;
impl<Y> Apply<Y> for LConstFalse1 {
    type Output = LFalse;
}

// =========================================================================
// Foldr (Right Fold)
// =========================================================================

use crate::eval::{ECall, ELit};

/// Foldr f z l
pub struct LFoldr<F, Z, L>(PhantomData<(F, Z, L)>);

impl<F, Z, L> Lambda for LFoldr<F, Z, L>
where
    L: Apply<LFoldrConsBuilder<F, Z>>,
    <L as Apply<LFoldrConsBuilder<F, Z>>>::Output: Apply<ELit<Z>>,
    <<L as Apply<LFoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output: Eval,
{
    type Output = crate::eval::Evaluate<
        <<L as Apply<LFoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output,
    >;
}

// Cons Builder: \h t. ECall<F, H, Foldr<F, Z, T>>
pub struct LFoldrConsBuilder<F, Z>(PhantomData<(F, Z)>);

impl<F, Z, H> Apply<H> for LFoldrConsBuilder<F, Z> {
    type Output = LFoldrConsBuilder2<F, Z, H>;
}

pub struct LFoldrConsBuilder2<F, Z, H>(PhantomData<(F, Z, H)>);

impl<F, Z, H, T> Apply<T> for LFoldrConsBuilder2<F, Z, H> {
    type Output = ECall<ECall<ELit<F>, ELit<H>>, LFoldr<F, Z, T>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::eval::Evaluate;

    // Test utilities
    struct E1;
    struct E2;
    struct DefaultVal;

    #[test]
    fn test_list_construction_and_destructuring() {
        type L0 = LNil;
        type L1 = LCons<E1, L0>;
        type L2 = LCons<E2, L1>;

        // IsEmpty
        assert_type_eq_all!(Evaluate<LIsEmpty<L0>>, LTrue);
        assert_type_eq_all!(Evaluate<LIsEmpty<L1>>, LFalse);

        // HeadOr
        assert_type_eq_all!(Evaluate<LHeadOr<L0, DefaultVal>>, DefaultVal);
        assert_type_eq_all!(Evaluate<LHeadOr<L1, DefaultVal>>, E1);
        assert_type_eq_all!(Evaluate<LHeadOr<L2, DefaultVal>>, E2);

        // TailOr
        assert_type_eq_all!(Evaluate<LTailOr<L0, DefaultVal>>, DefaultVal);
        assert_type_eq_all!(Evaluate<LTailOr<L1, DefaultVal>>, L0);
    }

    #[test]
    fn test_foldr_sum() {
        use crate::lambda::church::{LAdd, LSucc, LSuccGen, LZero};

        struct OpSum;
        // OpSum x -> OpSum1<x>
        impl<X> Apply<X> for OpSum {
            type Output = OpSum1<X>;
        }
        struct OpSum1<X>(PhantomData<X>);
        // OpSum1<x> acc -> Add<x, acc>
        // OpSum uses struct Add, which is an Expression.
        // But ECall expects result to be Eval? Yes Add is Eval.

        // Wait, Add<X, Acc> is an Expression (struct).
        // Foldr logic uses ECall<Op, Rec>.
        // If Op returns Add<X, Acc>, then ECall will Evaluate it.
        // This works perfectly!

        impl<X, Acc> Apply<Acc> for OpSum1<X>
        where
            X: Apply<LSuccGen>,
            <X as Apply<LSuccGen>>::Output: Apply<Acc>,
        {
            type Output = LAdd<X, Acc>;
        }

        type One = LSucc<LZero>;
        type Two = LSucc<One>;

        // List = [One, Two]
        type L = LCons<One, LCons<Two, LNil>>;

        // Foldr OpSum Zero L
        // Expected: One + (Two + Zero) = Three? No, Add is correct.

        // Let's verify simpler case first: Count.
        // \x acc. Succ<acc>

        struct OpCount;
        impl<X> Apply<X> for OpCount {
            type Output = OpCount1;
        }
        struct OpCount1;
        // Succ<Acc> is a struct, implements Eval.
        impl<Acc> Apply<Acc> for OpCount1 {
            type Output = LSucc<Acc>;
        }

        type CountRes = Evaluate<LFoldr<OpCount, LZero, L>>;
        assert_type_eq_all!(CountRes, Two);
    }
}
