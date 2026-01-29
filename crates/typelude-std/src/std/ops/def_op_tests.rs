//! **Tests for def_op! Procedural Macro**
//!
//! Unit tests for the def_op! procedural macro functionality (AST-only).

use std::ops::Add;

use static_assertions::assert_type_eq_all;
use typelude_macros::def_op;
use typelude_std::core::Evaluate;
use typenum::{U1, U2, U3, U5, U6, U12, U42};

def_op! {
    /// Test: Addition operation
    name: TestAddDef,
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
    use typelude_std::core::ELit;
    // Evaluate computes result
    type Result = Evaluate<TestEAdd<ELit<U1>, ELit<U2>>>;
    assert_type_eq_all!(Result, U3);
}
def_op! {
    /// Test: Identity operation (unary)
    name: TestIdDef,
    args: (T),
    ast: TestEId {
        where: [],
        type Output = Evaluate<T>
    }
}

#[test]
fn test_ast_pattern_unary() {
    use typelude_std::core::ELit;
    type Result = Evaluate<TestEId<ELit<U42>>>;
    assert_type_eq_all!(Result, U42);
}
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
    name: TestMulDef,
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
    use typelude_std::core::ELit;
    type R1 = Evaluate<TestEMul<ELit<U3>, ELit<U2>>>;
    assert_type_eq_all!(R1, U6);

    type R2 = Evaluate<TestEMul<ELit<typenum::U4>, ELit<U3>>>;
    assert_type_eq_all!(R2, U12);
}
#[test]
fn test_nested_evaluation() {
    use typelude_std::core::ELit;
    // (1 + 2) + 3 = 6
    type Inner = TestEAdd<ELit<U1>, ELit<U2>>;
    type Outer = TestEAdd<Inner, ELit<U3>>;
    type Result = Evaluate<Outer>;
    assert_type_eq_all!(Result, U6);

    // 2 + (1 + 2) = 5
    type Outer2 = TestEAdd<ELit<U2>, Inner>;
    type Result2 = Evaluate<Outer2>;
    assert_type_eq_all!(Result2, U5);
}
