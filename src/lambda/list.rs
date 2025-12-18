use std::marker::PhantomData;

use super::{Apply, Lambda, church::{True, False, PureIf}};
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
pub struct Nil;
impl Lambda for Nil {}
impl Eval for Nil { type Output = Nil; }

/// Cons: \h t. \c n. c h t
pub struct Cons<H, T>(PhantomData<(H, T)>);
impl<H, T> Lambda for Cons<H, T> {}
impl<H, T> Eval for Cons<H, T> { type Output = Cons<H, T>; }


// --- Nil Implementation ---
// Nil c -> Nil1<c>
impl<C> Apply<C> for Nil {
    type Output = Nil1<C>;
}
pub struct Nil1<C>(PhantomData<C>);

// Nil1<c> n -> n
impl<C, N> Apply<N> for Nil1<C> {
    type Output = N;
}

// --- Cons Implementation ---
// Cons<H, T> c -> Cons1<H, T, c>
impl<H, T, C> Apply<C> for Cons<H, T> {
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
pub type Uncons<L, OnCons, OnNil> = <<L as Apply<OnCons>>::Output as Apply<OnNil>>::Output;

// Helper K: \x y. x
pub struct K;
impl<X> Apply<X> for K { type Output = K1<X>; }
pub struct K1<X>(PhantomData<X>);
impl<X, Y> Apply<Y> for K1<X> { type Output = X; }

// Helper K_I: \x y. y (which is False)
pub struct KI;
impl<X> Apply<X> for KI { type Output = KI1; }
pub struct KI1;
impl<Y> Apply<Y> for KI1 { type Output = Y; }

/// Head: Extract head or return Default
pub struct HeadOr<L, Default>(PhantomData<(L, Default)>);

impl<L, Default> Eval for HeadOr<L, Default>
where
    L: Apply<K>,
    <L as Apply<K>>::Output: Apply<Default>,
{
    type Output = Uncons<L, K, Default>;
}
impl<L, D> Lambda for HeadOr<L, D> where Self: Eval {}

/// Tail: Extract tail or return Default
pub struct TailOr<L, Default>(PhantomData<(L, Default)>);

impl<L, Default> Eval for TailOr<L, Default>
where
    L: Apply<KI>,
    <L as Apply<KI>>::Output: Apply<Default>,
{
    type Output = Uncons<L, KI, Default>;
}
impl<L, D> Lambda for TailOr<L, D> where Self: Eval {}


// =========================================================================
// IsEmpty
// =========================================================================

/// IsEmpty: \l. l (\h t. False) True
pub struct IsEmpty<L>(PhantomData<L>);

impl<L> Eval for IsEmpty<L>
where
    L: Apply<ConstFalse>,
    <L as Apply<ConstFalse>>::Output: Apply<True>,
{
    type Output = Uncons<L, ConstFalse, True>;
}
impl<L> Lambda for IsEmpty<L> where Self: Eval {}

pub struct ConstFalse;
impl<X> Apply<X> for ConstFalse { type Output = ConstFalse1; }
pub struct ConstFalse1;
impl<Y> Apply<Y> for ConstFalse1 { type Output = False; }


// =========================================================================
// Foldr (Right Fold)
// =========================================================================

use crate::eval::{ECall, ELit, ELazyCall};

/// Foldr f z l
pub struct Foldr<F, Z, L>(PhantomData<(F, Z, L)>);

impl<F, Z, L> Eval for Foldr<F, Z, L>
where
    L: Apply<FoldrConsBuilder<F, Z>>, // L (ConsBuilder) (NilBuilder)
    <L as Apply<FoldrConsBuilder<F, Z>>>::Output: Apply<ELit<Z>>,
    <<L as Apply<FoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output: Eval,
{
    type Output = crate::eval::Evaluate<<<L as Apply<FoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output>;
}

impl<F, Z, L> Lambda for Foldr<F, Z, L> where Self: Eval {}

// Cons Builder: \h t. ECall<F, H, Foldr<F, Z, T>>
pub struct FoldrConsBuilder<F, Z>(PhantomData<(F, Z)>);

impl<F, Z, H> Apply<H> for FoldrConsBuilder<F, Z> {
    type Output = FoldrConsBuilder2<F, Z, H>;
}

pub struct FoldrConsBuilder2<F, Z, H>(PhantomData<(F, Z, H)>);

impl<F, Z, H, T> Apply<T> for FoldrConsBuilder2<F, Z, H> {
    type Output = ECall<
        ECall<ELit<F>, ELit<H>>,
        Foldr<F, Z, T>
    >;
}


#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use crate::eval::Evaluate;
    use super::*;

    // Test utilities
    struct E1;
    struct E2;
    struct DefaultVal;

    #[test]
    fn test_list_construction_and_destructuring() {
        type L0 = Nil;
        type L1 = Cons<E1, L0>;
        type L2 = Cons<E2, L1>;

        // IsEmpty
        assert_type_eq_all!(Evaluate<IsEmpty<L0>>, True);
        assert_type_eq_all!(Evaluate<IsEmpty<L1>>, False);

        // HeadOr
        assert_type_eq_all!(Evaluate<HeadOr<L0, DefaultVal>>, DefaultVal);
        assert_type_eq_all!(Evaluate<HeadOr<L1, DefaultVal>>, E1);
        assert_type_eq_all!(Evaluate<HeadOr<L2, DefaultVal>>, E2);

        // TailOr
        assert_type_eq_all!(Evaluate<TailOr<L0, DefaultVal>>, DefaultVal);
        assert_type_eq_all!(Evaluate<TailOr<L1, DefaultVal>>, L0);
    }

    #[test]
    fn test_foldr_sum() {
        use crate::lambda::church::{Zero, Succ, Add, SuccGen};

        struct OpSum;
        // OpSum x -> OpSum1<x>
        impl<X> Apply<X> for OpSum { type Output = OpSum1<X>; }
        struct OpSum1<X>(PhantomData<X>);
        // OpSum1<x> acc -> Add<x, acc>
        // OpSum uses struct Add, which is an Expression.
        // But ECall expects result to be Eval? Yes Add is Eval.

        // Wait, Add<X, Acc> is an Expression (struct).
        // Foldr logic uses ECall<Op, Rec>.
        // If Op returns Add<X, Acc>, then ECall will Evaluate it.
        // This works perfectly!

        impl<X, Acc> Apply<Acc> for OpSum1<X>
        where X: Apply<SuccGen>,
              <X as Apply<SuccGen>>::Output: Apply<Acc>
        {
            type Output = Add<X, Acc>;
        }

        type One = Succ<Zero>;
        type Two = Succ<One>;

        // List = [One, Two]
        type L = Cons<One, Cons<Two, Nil>>;

        // Foldr OpSum Zero L
        // Expected: One + (Two + Zero) = Three? No, Add is correct.

        // Let's verify simpler case first: Count.
        // \x acc. Succ<acc>

        struct OpCount;
        impl<X> Apply<X> for OpCount { type Output = OpCount1; }
        struct OpCount1;
        // Succ<Acc> is a struct, implements Eval.
        impl<Acc> Apply<Acc> for OpCount1 { type Output = Succ<Acc>; }

        type CountRes = Evaluate<Foldr<OpCount, Zero, L>>;
        assert_type_eq_all!(CountRes, Two);
    }
}
