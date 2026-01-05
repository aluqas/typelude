use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use super::{
    LApp, Lambda,
    church::{LFalse, LTrue},
    traits::{LList, LTerm},
};
// Macro to implement Eval for terms (they evaluate to themselves)
macro_rules! impl_eval_term {
    ($($t:ty),*) => {
        $(
            impl Eval for $t {
                type Output = $t;
            }
        )*
    };
}

/// Nil: \c n. n
pub struct LNil;
impl Lambda for LNil {
    type Output = LNil;
}
impl LTerm for LNil {}
impl LList for LNil {}
impl_eval_term!(LNil);

/// Cons: \h t. \c n. c h t
pub struct LCons;
impl Lambda for LCons {
    type Output = LCons;
}
impl_eval_term!(LCons);

// Cons H -> Cons1<H>
pub struct LCons1<H>(PhantomData<H>);
impl<H> Lambda for LCons1<H> {
    type Output = LCons1<H>;
}
impl<H> Eval for LCons1<H> {
    type Output = Self;
}

impl<H> Lambda for LApp<LCons, H>
where
    H: Eval,
{
    type Output = LCons1<Evaluate<H>>;
}

// Cons1<H> T -> Cons2<H, T> (The list value)
pub struct LCons2<H, T>(PhantomData<(H, T)>);
impl<H, T> Lambda for LCons2<H, T> {
    type Output = LCons2<H, T>;
}
impl<H, T> LTerm for LCons2<H, T> {}
impl<H, T> LList for LCons2<H, T> {} // Only if T is list? Not necessarily for encoding, but good for marking.
impl<H, T> Eval for LCons2<H, T> {
    type Output = Self;
}

impl<H, T> Lambda for LApp<LCons1<H>, T>
where
    T: Eval,
{
    type Output = LCons2<H, Evaluate<T>>;
}

// --- Nil Implementation ---
// Nil c -> Nil1<c>
impl<C> Lambda for LApp<LNil, C>
where
    C: Eval,
{
    type Output = Nil1<Evaluate<C>>;
}
pub struct Nil1<C>(PhantomData<C>);
impl<C> Lambda for Nil1<C> {
    type Output = Nil1<C>;
}
impl<C> Eval for Nil1<C> {
    type Output = Self;
}

// Nil1<c> n -> n
impl<C, N> Lambda for LApp<Nil1<C>, N>
where
    N: Eval,
{
    type Output = Evaluate<N>;
}

// --- Cons Implementation ---
// Cons2<H, T> C -> ConsApply1<H, T, C>
impl<H, T, C> Lambda for LApp<LCons2<H, T>, C>
where
    H: Eval,
    T: Eval,
    C: Eval,
{
    type Output = ConsApply1<H, T, Evaluate<C>>;
}

pub struct ConsApply1<H, T, C>(PhantomData<(H, T, C)>);
impl<H, T, C> Lambda for ConsApply1<H, T, C> {
    type Output = ConsApply1<H, T, C>;
}
impl<H, T, C> Eval for ConsApply1<H, T, C> {
    type Output = Self;
}

// ConsApply1<H, T, C> N -> C H T
impl<H, T, C, N> Lambda for LApp<ConsApply1<H, T, C>, N>
where
    H: Eval,
    T: Eval,
    C: Eval,
    N: Eval,
    LApp<C, H>: Lambda,
    LApp<<LApp<C, H> as Lambda>::Output, T>: Lambda,
{
    type Output = <LApp<<LApp<C, H> as Lambda>::Output, T> as Lambda>::Output;
}
/// Uncons l on_cons on_nil
// LApp<LApp<L, OnCons>, OnNil>
pub type LPureUncons<L, OnCons, OnNil> = Evaluate<LApp<LApp<L, OnCons>, OnNil>>;

// Helper K: \x y. x
pub struct K;
impl Lambda for K {
    type Output = K;
}
impl_eval_term!(K);

impl<X> Lambda for LApp<K, X>
where
    X: Eval,
{
    type Output = K1<Evaluate<X>>;
}
pub struct K1<X>(PhantomData<X>);
impl<X> Lambda for K1<X> {
    type Output = K1<X>;
}
impl<X> Eval for K1<X> {
    type Output = Self;
}

impl<X, Y> Lambda for LApp<K1<X>, Y>
where
    Y: Eval,
    X: Eval,
{
    type Output = Evaluate<X>;
}

// Helper K_I: \x y. y (False)
// Use LFalse from bool.rs?
// But list.rs shouldn't depend on bool.rs details ideally, but standard
// combinators are standard. Let's reuse LFalse.

/// HeadOr: \l d. l K d
pub struct LHeadOr;
impl Lambda for LHeadOr {
    type Output = LHeadOr;
}
impl_eval_term!(LHeadOr);

impl<L> Lambda for LApp<LHeadOr, L>
where
    L: Eval,
{
    type Output = LHeadOr1<Evaluate<L>>;
}
pub struct LHeadOr1<L>(PhantomData<L>);
impl<L> Lambda for LHeadOr1<L> {
    type Output = LHeadOr1<L>;
}
impl<L> Eval for LHeadOr1<L> {
    type Output = Self;
}

impl<L, D> Lambda for LApp<LHeadOr1<L>, D>
where
    L: Eval,
    D: Eval,
    LApp<L, K>: Lambda,
    LApp<<LApp<L, K> as Lambda>::Output, D>: Lambda,
{
    type Output = <LApp<<LApp<L, K> as Lambda>::Output, D> as Lambda>::Output;
}

/// TailOr: \l d. l K_I d
pub struct LTailOr;
impl Lambda for LTailOr {
    type Output = LTailOr;
}
impl_eval_term!(LTailOr);

impl<L> Lambda for LApp<LTailOr, L>
where
    L: Eval,
{
    type Output = LTailOr1<Evaluate<L>>;
}
pub struct LTailOr1<L>(PhantomData<L>);
impl<L> Lambda for LTailOr1<L> {
    type Output = LTailOr1<L>;
}
impl<L> Eval for LTailOr1<L> {
    type Output = Self;
}

impl<L, D> Lambda for LApp<LTailOr1<L>, D>
where
    L: Eval,
    D: Eval,
    LApp<L, LFalse>: Lambda,
    LApp<<LApp<L, LFalse> as Lambda>::Output, D>: Lambda,
{
    type Output = <LApp<<LApp<L, LFalse> as Lambda>::Output, D> as Lambda>::Output;
}
/// IsEmpty: \l. l (\h t. False) True
pub struct LIsEmpty;
impl Lambda for LIsEmpty {
    type Output = LIsEmpty;
}
impl_eval_term!(LIsEmpty);

impl<L> Lambda for LApp<LIsEmpty, L>
where
    L: Eval,
    LApp<L, LConstFalse>: Lambda,
    LApp<<LApp<L, LConstFalse> as Lambda>::Output, LTrue>: Lambda,
{
    type Output = <LApp<<LApp<L, LConstFalse> as Lambda>::Output, LTrue> as Lambda>::Output;
}

pub struct LConstFalse;
impl Lambda for LConstFalse {
    type Output = LConstFalse;
}
impl_eval_term!(LConstFalse);

impl<X> Lambda for LApp<LConstFalse, X>
where
    X: Eval,
{
    type Output = LConstFalse1;
}
pub struct LConstFalse1;
impl Lambda for LConstFalse1 {
    type Output = LConstFalse1;
}
impl_eval_term!(LConstFalse1);

impl<Y> Lambda for LApp<LConstFalse1, Y>
where
    Y: Eval,
{
    type Output = LFalse;
}
/// Foldr f z l
pub struct LFoldr;
impl Lambda for LFoldr {
    type Output = LFoldr;
}
impl_eval_term!(LFoldr);

// Foldr F -> Foldr1<F>
impl<F> Lambda for LApp<LFoldr, F>
where
    F: Eval,
{
    type Output = LFoldr1<Evaluate<F>>;
}
pub struct LFoldr1<F>(PhantomData<F>);
impl<F> Lambda for LFoldr1<F> {
    type Output = LFoldr1<F>;
}
impl<F> Eval for LFoldr1<F> {
    type Output = Self;
}

// Foldr1<F> Z -> Foldr2<F, Z>
impl<F, Z> Lambda for LApp<LFoldr1<F>, Z>
where
    Z: Eval,
{
    type Output = LFoldr2<F, Evaluate<Z>>;
}
pub struct LFoldr2<F, Z>(PhantomData<(F, Z)>);
impl<F, Z> Lambda for LFoldr2<F, Z> {
    type Output = LFoldr2<F, Z>;
}
impl<F, Z> Eval for LFoldr2<F, Z> {
    type Output = Self;
}

// Foldr2<F, Z> L -> Result
impl<F, Z, L> Lambda for LApp<LFoldr2<F, Z>, L>
where
    F: Eval + Clone,
    Z: Eval + Clone,
    L: Eval,
    // L (ConsBuilder F Z) Z
    LApp<L, LFoldrConsBuilder<F, Z>>: Lambda,
    LApp<<LApp<L, LFoldrConsBuilder<F, Z>> as Lambda>::Output, Z>: Lambda,
{
    type Output =
        <LApp<<LApp<L, LFoldrConsBuilder<F, Z>> as Lambda>::Output, Z> as Lambda>::Output;
}

// Cons Builder: \h t. F h (Foldr F Z t)
pub struct LFoldrConsBuilder<F, Z>(PhantomData<(F, Z)>);
impl<F, Z> Lambda for LFoldrConsBuilder<F, Z> {
    type Output = LFoldrConsBuilder<F, Z>;
}
impl<F, Z> Eval for LFoldrConsBuilder<F, Z> {
    type Output = Self;
}

// Builder<F, Z> H -> Builder1<F, Z, H>
impl<F, Z, H> Lambda for LApp<LFoldrConsBuilder<F, Z>, H>
where
    F: Eval,
    Z: Eval,
    H: Eval,
{
    type Output = LFoldrConsBuilder1<F, Z, Evaluate<H>>;
}

pub struct LFoldrConsBuilder1<F, Z, H>(PhantomData<(F, Z, H)>);
impl<F, Z, H> Lambda for LFoldrConsBuilder1<F, Z, H> {
    type Output = LFoldrConsBuilder1<F, Z, H>;
}
impl<F, Z, H> Eval for LFoldrConsBuilder1<F, Z, H> {
    type Output = Self;
}

// Builder1<F, Z, H> T -> F H (Foldr F Z T)
impl<F, Z, H, T> Lambda for LApp<LFoldrConsBuilder1<F, Z, H>, T>
where
    F: Eval + Clone, Z: Eval + Clone, H: Eval, T: Eval,
    // Recurse: Foldr F Z T
    LApp<LFoldr, F>: Lambda, // Foldr1
    LApp<<LApp<LFoldr, F> as Lambda>::Output, Z>: Lambda, // Foldr2
    LApp<<LApp<<LApp<LFoldr, F> as Lambda>::Output, Z> as Lambda>::Output, T>: Lambda, // Result

    // Apply F H
    LApp<F, H>: Lambda,
    // Apply (F H) to Recurse Result
    LApp<<LApp<F, H> as Lambda>::Output, <LApp<<LApp<<LApp<LFoldr, F> as Lambda>::Output, Z> as Lambda>::Output, T> as Lambda>::Output>: Lambda,
{
    type Output = <LApp<
        <LApp<F, H> as Lambda>::Output,
        <LApp<<LApp<<LApp<LFoldr, F> as Lambda>::Output, Z> as Lambda>::Output, T> as Lambda>::Output
    > as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    // Test utilities
    #[derive(Clone)]
    struct E1;
    impl Lambda for E1 {
        type Output = E1;
    }
    impl Eval for E1 {
        type Output = Self;
    }
    #[derive(Clone)]
    struct E2;
    impl Lambda for E2 {
        type Output = E2;
    }
    impl Eval for E2 {
        type Output = Self;
    }
    #[derive(Clone)]
    struct DefaultVal;
    impl Lambda for DefaultVal {
        type Output = DefaultVal;
    }
    impl Eval for DefaultVal {
        type Output = Self;
    }

    #[test]
    fn test_list_construction_and_destructuring() {
        type L0 = LNil;
        type L1 = App<App<LCons, E1>, L0>; // Cons E1 Nil
        type L2 = App<App<LCons, E2>, L1>; // Cons E2 (Cons E1 Nil)

        // IsEmpty
        assert_type_eq_all!(App<LIsEmpty, L0>, LTrue);
        assert_type_eq_all!(App<LIsEmpty, L1>, LFalse);

        // HeadOr
        assert_type_eq_all!(App<App<LHeadOr, L0>, DefaultVal>, DefaultVal);
        assert_type_eq_all!(App<App<LHeadOr, L1>, DefaultVal>, E1);
        assert_type_eq_all!(App<App<LHeadOr, L2>, DefaultVal>, E2);

        // TailOr
        assert_type_eq_all!(App<App<LTailOr, L0>, DefaultVal>, DefaultVal);
        assert_type_eq_all!(App<App<LTailOr, L1>, DefaultVal>, L0);
    }
}
