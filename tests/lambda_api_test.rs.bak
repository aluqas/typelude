#![recursion_limit = "512"]

//! Test verifying the usability of the Lambda API Facade.

use static_assertions::assert_type_eq_all;
use typelude::{Evaluate, lambda::*};

#[test]
fn test_prelude_access() {
    // Aliases
    type App<F, A> = <F as Apply<A>>::Output;

    // Helper to normalize Church (LAdd2/LAdd3) to Structural (LSucc)
    type Normalize<N> = <<N as Apply<LSuccGen>>::Output as Apply<LZero>>::Output;

    // 1. Numerals
    type One = LSucc<LZero>;
    type Two = LSucc<One>;

    // LAdd One Two
    type ThreeExpr = App<App<LAdd, One>, Two>;
    type Three = Evaluate<ThreeExpr>; // Reduced to canonical function form (LAdd2)

    // 2. Booleans
    type T = LTrue;
    // LIf T One Two
    // LIf struct is already the expression `If P T E`
    type Check = LIf<T, One, Two>;

    // 3. Lists
    type List1 = LCons<One, LCons<Two, LNil>>;
    // LFoldr LAdd LZero List1
    type SumExpr = LFoldr<LAdd, LZero, List1>;
    type Sum = Evaluate<SumExpr>;

    // Verify
    // 3 = 1 + 2
    assert_type_eq_all!(Normalize<Three>, LSucc<LSucc<LSucc<LZero>>>);

    // If True 1 2 = 1
    assert_type_eq_all!(Evaluate<Check>, One);

    // Foldr (+) 0 [1, 2] = 1 + (2 + 0) = 3
    assert_type_eq_all!(Normalize<Sum>, LSucc<LSucc<LSucc<LZero>>>);
}

#[test]
fn test_monad_prelude() {
    // Verify monad types are accessible
    // LId<T> requires a type argument.
    type S = LState<LId<LZero>>;
    type E = LLeft<LZero>;

    // Just type check existence
    let _ = std::marker::PhantomData::<(S, E)>;
}
