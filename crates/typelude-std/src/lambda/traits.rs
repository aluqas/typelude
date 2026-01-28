//! **Lambda Traits**
//!
//! Traits and AST nodes for type-level lambda calculus.

use std::marker::PhantomData;

use typelude_core::Evaluate;

/// Macro to implement Eval for a Lambda value type.
/// Usage: `impl_eval_for_lambda!(MyStruct);`
#[macro_export]
macro_rules! impl_eval_for_lambda {
    ($t:ty) => {
        impl $crate::typelude_core::Eval for $t {
            type Output = $t;
        }
    };
}

/// Macro for generic Lambda value types.
/// Usage: `impl_eval_for_lambda_generic!(MyStruct, [T, U]);`
#[macro_export]
macro_rules! impl_eval_for_lambda_generic {
    ($t:ident, [$($p:ident),+]) => {
        impl<$($p),+> $crate::typelude_core::Eval for $t<$($p),+>
        {
            type Output = $t<$($p),+>;
        }
    };
}
/// **Lambda Application**: `LApp<F, A>`
///
/// Represents the application of function `F` to argument `A`.
/// This struct is used with the `Eval` pattern.
pub struct LApp<F, A>(PhantomData<(F, A)>);
/// Type-level application result: `Apply<F, A> = Evaluate<LApp<F, A>>`.
// pub type Apply<F, A> = Evaluate<LApp<F, A>>; // Deprecated/Removed to avoid conflict with core::Apply trait
/// Bind Trait: `m >>= f`
pub trait LBind<F> {
    type Output;
}
/// Base trait for all Lambda Terms.
pub trait LTerm {}

/// Marker for Church Booleans.
pub trait LBool: LTerm {}

/// Marker for Church Numerals.
pub trait LNat: LTerm {}

/// Marker for Church Lists.
pub trait LList: LTerm {}
