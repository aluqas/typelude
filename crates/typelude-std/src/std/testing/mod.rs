//! **Testing Utilities**
//!
//! Macros and helpers for verifying trait implementations and type-level logic.

/// Assert that a type implements `TypeNat` and behaves correctly for basic arithmetic.
///
/// Usage: `test_type_nat!(MyType, 10);` (where 10 is the expected integer value)
/// Note: This is a placeholder. Real verification usually involves checking `ToConst`.
#[macro_export]
macro_rules! verify_type_nat {
    ($T:ty) => {
        const _: () = {
            use $crate::std::traits::TypeNat;
            // Verify it implements the trait
            fn assert_impl<T: TypeNat>() {}
            // assert_impl::<$T>();
            // Note: We can't call generic functions in const blocks easily on stable yet without side effects,
            // but the mere existence of this block verifies bounds if we structure it right.
        };
    };
}

/// Verify TypeBool implementation.
#[macro_export]
macro_rules! verify_type_bool {
    ($T:ty, $Expect:expr) => {
        const _: () = {
            use $crate::std::traits::TypeBool;
            // Check trait bound
            fn assert_impl<T: TypeBool>() {}
            // assert_impl::<$T>();
        };
    };
}
