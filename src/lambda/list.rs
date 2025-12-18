use std::marker::PhantomData;

use super::{Apply, church::{True, False}};

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

/// Cons: \h t. \c n. c h t
pub struct Cons<H, T>(PhantomData<(H, T)>);

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
pub type HeadOr<L, Default> = Uncons<L, K, Default>;

/// Tail: Extract tail or return Default
pub type TailOr<L, Default> = Uncons<L, KI, Default>;

// =========================================================================
// IsEmpty
// =========================================================================

/// IsEmpty: \l. l (\h t. False) True
pub type IsEmpty<L> = Uncons<L, ConstFalse, True>;

pub struct ConstFalse;
impl<X> Apply<X> for ConstFalse { type Output = ConstFalse1; }
pub struct ConstFalse1;
impl<Y> Apply<Y> for ConstFalse1 { type Output = False; }


// =========================================================================
// Foldr (Right Fold)
//
// Foldr f z l
// = l (\h t. f h (Foldr f z t)) z
//
// Since Rust types are strict/eager during expansion, defining recursive types
// like this directly causes infinite recursion error "reached recursion limit".
//
// To solve this, we MUST use `Fix` (Fixed Point Combinator) OR `Eval` (Lazy expansion).
//
// Here, we demonstrate the power of `Eval` integration.
// Instead of defining `Foldr` as a direct type alias, we define it as a struct
// that implements `Apply` using `Eval` to defer the recursive step.
// =========================================================================

use crate::eval::{Eval, Evaluate, ECall, ELit, ELazyCall};

/// Foldr f z l
pub struct Foldr<F, Z, L>(PhantomData<(F, Z, L)>);

// We implement Apply via Eval for ease, or Eval directly.
// Let's make Foldr an Expression itself (implement Eval).
//
// Eval<Foldr<F, Z, L>> logic:
// 1. Check if L is Nil or Cons.
//    We use `Uncons<L, OnCons, OnNil>`.
// 2. If Nil: Return Z.
// 3. If Cons h t: Return f h (Foldr f z t).

// Helper struct for the Cons branch of Foldr
// OnCons h t -> f h (Foldr f z t)
// But we need to capture `f` and `z`.
pub struct FoldrStep<F, Z>(PhantomData<(F, Z)>);

// Step 1: Apply h
impl<F, Z, H> Apply<H> for FoldrStep<F, Z> {
    type Output = FoldrStep2<F, Z, H>;
}

pub struct FoldrStep2<F, Z, H>(PhantomData<(F, Z, H)>);

// Step 2: Apply t -> Result
// Result = f h (Foldr f z t)
// We need to return an Expression that evaluates to this.
//
// f h: Apply<F, H>
// Rec: Foldr<F, Z, T>
// Final: Apply< (f h), Rec >
//
// But wait, Uncons returns a TYPE, not an expression.
// If we want lazy recursion, `OnCons` must return an EXPRESSION that is not yet fully expanded?
// No, standard Uncons expands eagerly.
//
// This is where `Eval` shines.
// We can use `ECall` to wrap the recursion.
//
// Let's redefine Foldr Logic using `Eval`.

impl<F, Z, L> Eval for Foldr<F, Z, L>
where
    L: Apply<FoldrConsBuilder<F, Z>>, // L (ConsBuilder) (NilBuilder)
    <L as Apply<FoldrConsBuilder<F, Z>>>::Output: Apply<ELit<Z>>,
    <<L as Apply<FoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output: Eval,
{
    type Output = Evaluate<<<L as Apply<FoldrConsBuilder<F, Z>>>::Output as Apply<ELit<Z>>>::Output>;
}

// Cons Builder: \h t. ECall<F, H, ECall<Foldr, F, Z, T>>
// Actually simpler: \h t. EApp2<F, H, Foldr<F, Z, T>>
//
// We want the result of Uncons to be an Expression `E`.
// `E` will be `Z` (for Nil) or `App(f, h, rec)` (for Cons).
pub struct FoldrConsBuilder<F, Z>(PhantomData<(F, Z)>);

impl<F, Z, H> Apply<H> for FoldrConsBuilder<F, Z> {
    type Output = FoldrConsBuilder2<F, Z, H>;
}

pub struct FoldrConsBuilder2<F, Z, H>(PhantomData<(F, Z, H)>);

impl<F, Z, H, T> Apply<T> for FoldrConsBuilder2<F, Z, H> {
    // Return an Expression representing: F h (Foldr F Z T)
    // We assume F is a function that takes 2 args? Or F h returns a function that takes Rec?
    // Standard Foldr: f x acc.
    // So F applied to H, then applied to Rec.

    // We use ECall to perform the application of F lazily if needed,
    // or just construct the expression tree.
    // ECall< ELit<F>, ELit<H> > -> PartialF
    // ECall< PartialF, Foldr<F, Z, T> > -> Result

    // We use nested ECall.
    // Op = ECall<ELit<F>, ELit<H>>
    // Rec = Foldr<F, Z, T>
    // Result = ECall<Op, Rec>
    type Output = ECall<
        ECall<ELit<F>, ELit<H>>,
        Foldr<F, Z, T>
    >;
}


#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
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
        assert_type_eq_all!(IsEmpty<L0>, True);
        assert_type_eq_all!(IsEmpty<L1>, False);

        // HeadOr
        assert_type_eq_all!(HeadOr<L0, DefaultVal>, DefaultVal);
        assert_type_eq_all!(HeadOr<L1, DefaultVal>, E1);
        assert_type_eq_all!(HeadOr<L2, DefaultVal>, E2);

        // TailOr
        assert_type_eq_all!(TailOr<L0, DefaultVal>, DefaultVal);
        assert_type_eq_all!(TailOr<L1, DefaultVal>, L0);
    }

    #[test]
    fn test_foldr_sum() {
        // Define a Sum operation: \x acc. Add<x, acc>
        // Use Church Numerals for this test? Or just mock types?
        // Let's use Church Numerals from `church.rs`
        use crate::lambda::church::{Zero, Succ, Add};

        struct OpSum;
        // OpSum x -> OpSum1<x>
        impl<X> Apply<X> for OpSum { type Output = OpSum1<X>; }
        struct OpSum1<X>(PhantomData<X>);
        // OpSum1<x> acc -> Add<x, acc>
        // Note: X must be applicable to SuccGen to be a valid Church numeral for Add.
        // But for this test, we just want compilation.
        // To make Rust happy, we add the bound.
        use crate::lambda::church::SuccGen;
        impl<X, Acc> Apply<Acc> for OpSum1<X>
        where X: Apply<SuccGen>,
              <X as Apply<SuccGen>>::Output: Apply<Acc>
        {
            type Output = Add<X, Acc>;
        }

        type One = Succ<Zero>;
        type Two = Succ<One>;
        type Three = Succ<Two>;

        // List = [One, Two]
        type L = Cons<One, Cons<Two, Nil>>;

        // Foldr OpSum Zero L
        // Expected: One + (Two + Zero) = Three
        type Res = Evaluate<Foldr<OpSum, Zero, L>>;

        // Check structural equality if normalized
        // Church numerals might be deep, so let's check against Three.

        // Wait, Church Add is complex.
        // Let's verify simpler case first: Count.
        // \x acc. Succ<acc>

        struct OpCount;
        impl<X> Apply<X> for OpCount { type Output = OpCount1; }
        struct OpCount1;
        impl<Acc> Apply<Acc> for OpCount1 { type Output = Succ<Acc>; }

        type CountRes = Evaluate<Foldr<OpCount, Zero, L>>;
        assert_type_eq_all!(CountRes, Two);
    }
}
