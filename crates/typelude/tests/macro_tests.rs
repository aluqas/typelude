#![recursion_limit = "128000"]
use typelude::{
    program,
    std::debug::trace::Trace,
    typenum::{U1, U2},
};

// Debug test to verify environment
#[test]
fn debug_environment_check() {
    // 1. Can we access typelude_std directly?
    use typelude_std::tyarray;
    type Arr1 = tyarray![U1];
    assert_eq!(<Arr1 as Trace>::fmt(), "[1]");

    // 2. Can we access typelude::tyarray?
    // Note: typelude::tyarray is re-exported from typelude_std
    type Arr2 = typelude::tyarray![U2];
    assert_eq!(<Arr2 as Trace>::fmt(), "[2]");
}

#[test]
fn test_stack_ops() {
    // Push, Dup, Swap, Drop
    type Res = program! {
        (push 1)
        (push 2)
        (dup)
        // Stack: [2, 2, 1]
        (swap)
        // Stack: [2, 2, 1] (Swap swaps top 2)
        (drop)
        // Stack: [2, 1]
    };
    // Expected: [2, 1]
    assert_eq!(<Res as Trace>::fmt(), "[2, 1]");
}

#[test]
fn test_arithmetic_ops() {
    // Add: 1 + 2 = 3
    type ResAdd = program! { push 1 push 2 add };
    assert_eq!(<ResAdd as Trace>::fmt(), "[3]");

    // Sub: 5 - 3 = 2
    // Stack order: push 3 first, then push 5 (top), sub computes top - second = 5 -
    // 3
    type ResSub = program! { push 3 push 5 sub };
    assert_eq!(<ResSub as Trace>::fmt(), "[2]");
}

#[test]
fn test_comparison_ops() {
    // Eq: 1 == 1 -> true
    type ResEq = program! { push 1 push 1 eq };
    assert_eq!(<ResEq as Trace>::fmt(), "[true]");

    // Eq: 1 == 2 -> false
    type ResEqFalse = program! { push 1 push 2 eq };
    assert_eq!(<ResEqFalse as Trace>::fmt(), "[false]");

    // Neq: 1 != 2 -> true
    type ResNeq = program! { push 1 push 2 neq };
    assert_eq!(<ResNeq as Trace>::fmt(), "[true]");

    // Lt: 1 < 2 -> true (push 2 first, then 1; lt computes top < second = 1 < 2)
    type ResLt = program! { push 2 push 1 lt };
    assert_eq!(<ResLt as Trace>::fmt(), "[true]");

    // Gt: 2 > 1 -> true (push 1 first, then 2; gt computes top > second = 2 > 1)
    type ResGt = program! { push 1 push 2 gt };
    assert_eq!(<ResGt as Trace>::fmt(), "[true]");
}

#[test]
fn test_logic_ops() {
    // And: true && true -> true (B1 is 1)
    type ResAnd = program! {
    push typelude::typenum::B1
    push typelude::typenum::B1
    and
    };
    assert_eq!(<ResAnd as Trace>::fmt(), "[true]");

    type ResOr = program! {
    push typelude::typenum::B0
    push typelude::typenum::B1
    or
    };
    assert_eq!(<ResOr as Trace>::fmt(), "[true]");

    type ResNot = program! {
    push typelude::typenum::B0
    not
    };
    assert_eq!(<ResNot as Trace>::fmt(), "[true]");
}

#[test]
fn test_control_intrinsics() {
    // Return
    type ResRet = program! { push 42 };
    assert_eq!(<ResRet as Trace>::fmt(), "[42]");
}

#[test]
fn test_variables() {
    // let, set, get
    type ResVar = program! {
        push 10
        let x
        // Stack: []
        push 5
        set x
        get x
    };
    assert_eq!(<ResVar as Trace>::fmt(), "[5]");
}

#[test]
fn test_control_flow_if() {
    // if true then 1 else 2
    type ResIfTrue = program! {
        push typelude::typenum::B1
        if ( push 1 ) ( push 2 )
    };
    assert_eq!(<ResIfTrue as Trace>::fmt(), "[1]");

    type ResIfFalse = program! {
        push typelude::typenum::B0
        if ( push 1 ) ( push 2 )
    };
    assert_eq!(<ResIfFalse as Trace>::fmt(), "[2]");
}

#[test]
fn test_control_flow_while() {
    // Simple test: While ( false ) ( ... ) -> should not run.
    type ResWhileZero = program! {
        push 1
        while ( push typelude::typenum::B0 ) ( push 0 )
    };
    assert_eq!(<ResWhileZero as Trace>::fmt(), "[1]");
}
