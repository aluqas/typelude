//! **Lambda Traits**
//!
//! Traits specific to pure functional programming context.

use std::marker::PhantomData;

use crate::eval::Eval;

// =========================================================================
// Lambda Trait
// =========================================================================

/// **Pure Lambda Term Trait**
///
/// Types implementing this trait represent pure lambda calculus terms.
pub trait Lambda {
    type Output;
}

/// Macro to implement Eval for a Lambda type.
/// Usage: `impl_eval_for_lambda!(MyStruct);`
#[macro_export]
macro_rules! impl_eval_for_lambda {
    ($t:ty) => {
        impl $crate::eval::Eval for $t {
            type Output = <$t as $crate::lambda::traits::Lambda>::Output;
        }
    };
}

/// Macro for generic Lambda types.
/// Usage: `impl_eval_for_lambda_generic!(MyStruct, [T, U]);`
#[macro_export]
macro_rules! impl_eval_for_lambda_generic {
    ($t:ident, [$($p:ident),+]) => {
        impl<$($p),+> $crate::eval::Eval for $t<$($p),+>
        where
            $t<$($p),+>: $crate::lambda::traits::Lambda,
        {
            type Output = <$t<$($p),+> as $crate::lambda::traits::Lambda>::Output;
        }
    };
}

// =========================================================================
// Application Struct (Eval Pattern)
// =========================================================================

/// **Lambda Application**: `LApp<F, A>`
///
/// Represents the application of function `F` to argument `A`.
/// This struct is used with the `Eval` pattern.
pub struct LApp<F, A>(PhantomData<(F, A)>);

// Implement Eval for LApp directly, effectively replacing the blanket impl for this specific type.
// Assuming LApp implements Lambda (which is usually where logic lives).
impl<F, A> Eval for LApp<F, A>
where
    Self: Lambda,
{
    type Output = <Self as Lambda>::Output;
}

// =========================================================================
// Bind Trait (Monad)
// =========================================================================

/// Bind Trait: `m >>= f`
pub trait LBind<F> {
    type Output;
}

// =========================================================================
// Kind System (Marker Traits)
// =========================================================================

/// Base trait for all Lambda Terms.
pub trait LTerm {}

/// Marker for Church Booleans.
pub trait LBool: LTerm {}

/// Marker for Church Numerals.
pub trait LNat: LTerm {}

/// Marker for Church Lists.
pub trait LList: LTerm {}
