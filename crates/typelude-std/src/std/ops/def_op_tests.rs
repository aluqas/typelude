//! **Tests for def_op! Procedural Macro**
//!
//! Unit tests for the def_op! procedural macro functionality.

use std::ops::Add;

use static_assertions::assert_type_eq_all;
use typelude_macros::def_op;
use typenum::{U1, U2, U3, U5, U6, U12, U42};

use crate::{
    eval::{Eval, Evaluate},
    model::traits::Apply,
};

// =============================================================================
// Test: AST Pattern - Basic Binary Operation
// =============================================================================

def_op! {
    /// Test: Addition operation
    name: TestOpAdd,
    args: (Lhs, Rhs),
    ast: TestEAdd {
        where: [
            Evaluate<Lhs>: Add<Evaluate<Rhs>>
        ],
        type Output = <Evaluate<Lhs> as Add<Evaluate<Rhs>>>::Output
    }
}

#[test]
fn test_ast_pattern_add() {
    // Apply returns AST
    type Ast = <TestOpAdd as Apply<(U1, U2)>>::Output;
    assert_type_eq_all!(Ast, TestEAdd<U1, U2>);

    // Evaluate computes result
    type Result = Evaluate<TestEAdd<U1, U2>>;
    assert_type_eq_all!(Result, U3);
}

// =============================================================================
// Test: AST Pattern - Unary Operation
// =============================================================================

def_op! {
    /// Test: Identity operation (unary)
    name: TestOpId,
    args: (T),
    ast: TestEId {
        where: [],
        type Output = Evaluate<T>
    }
}

#[test]
fn test_ast_pattern_unary() {
    type Ast = <TestOpId as Apply<U42>>::Output;
    assert_type_eq_all!(Ast, TestEId<U42>);

    type Result = Evaluate<TestEId<U42>>;
    assert_type_eq_all!(Result, U42);
}

// =============================================================================
// Test: Alias Pattern - Simple Alias
// =============================================================================

/// A simple wrapper AST for testing
pub struct TestWrap<T>(std::marker::PhantomData<T>);
impl<T: Eval> Eval for TestWrap<T> {
    type Output = Evaluate<T>;
}

def_op! {
    /// Test: Wrap operation (alias)
    name: TestOpWrap,
    args: (T),
    alias: TestWrap<T>
}

#[test]
fn test_alias_pattern() {
    // Apply directly returns the alias type
    type Ast = <TestOpWrap as Apply<U1>>::Output;
    assert_type_eq_all!(Ast, TestWrap<U1>);

    // Evaluate through the wrapper
    type Result = Evaluate<TestWrap<U1>>;
    assert_type_eq_all!(Result, U1);
}

// =============================================================================
// Test: Alias Pattern - Binary Alias
// =============================================================================

def_op! {
    /// Test: Add alias redirecting to TestEAdd
    name: TestOpAddAlias,
    args: (L, R),
    alias: TestEAdd<L, R>
}

#[test]
fn test_alias_pattern_binary() {
    type Ast = <TestOpAddAlias as Apply<(U5, U1)>>::Output;
    assert_type_eq_all!(Ast, TestEAdd<U5, U1>);

    type Result = Evaluate<Ast>;
    assert_type_eq_all!(Result, U6);
}

// =============================================================================
// Test: Complex Where Bounds
// =============================================================================

/// Custom Multiply trait for testing
pub trait TestMul<Rhs> {
    type Output;
}

impl TestMul<U2> for U3 {
    type Output = U6;
}

impl TestMul<U3> for typenum::U4 {
    type Output = U12;
}

def_op! {
    /// Test: Multiply operation with complex bounds
    name: TestOpMul,
    args: (Lhs, Rhs),
    ast: TestEMul {
        where: [
            Evaluate<Lhs>: TestMul<Evaluate<Rhs>>
        ],
        type Output = <Evaluate<Lhs> as TestMul<Evaluate<Rhs>>>::Output
    }
}

#[test]
fn test_complex_bounds() {
    type R1 = Evaluate<TestEMul<U3, U2>>;
    assert_type_eq_all!(R1, U6);

    type R2 = Evaluate<TestEMul<typenum::U4, U3>>;
    assert_type_eq_all!(R2, U12);
}

// =============================================================================
// Test: Nested Evaluation
// =============================================================================

#[test]
fn test_nested_evaluation() {
    // (1 + 2) + 3 = 6
    type Inner = TestEAdd<U1, U2>;
    type Outer = TestEAdd<Inner, U3>;
    type Result = Evaluate<Outer>;
    assert_type_eq_all!(Result, U6);

    // 2 + (1 + 2) = 5
    type Outer2 = TestEAdd<U2, Inner>;
    type Result2 = Evaluate<Outer2>;
    assert_type_eq_all!(Result2, U5);
}
