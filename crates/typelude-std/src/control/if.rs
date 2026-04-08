//! Conditional expression AST.

use core::marker::PhantomData;

/// Conditional expression AST.
///
/// Semantics may be provided either directly with `Eval` impls or via sugar
/// over `Apply<OpIf, ...>`.
///
/// # Examples
///
/// Direct `Eval` implementations remain supported for control-flow nodes.
///
/// ```ignore
/// use static_assertions::assert_type_eq_all;
/// use typelude_std::{Eval, Evaluate, If};
///
/// struct Yes;
/// struct No;
///
/// impl<Then, Else> Eval for If<Yes, Then, Else> {
///     type Output = Then;
/// }
///
/// impl<Then, Else> Eval for If<No, Then, Else> {
///     type Output = Else;
/// }
///
/// assert_type_eq_all!(Evaluate<If<Yes, u8, u16>>, u8);
/// assert_type_eq_all!(Evaluate<If<No, u8, u16>>, u16);
/// ```
pub struct If<Cond, Then, Else>(pub PhantomData<(Cond, Then, Else)>);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{Eval, Evaluate, If};

    struct Yes;
    struct No;

    impl<Then, Else> Eval for If<Yes, Then, Else> {
        type Output = Then;
    }

    impl<Then, Else> Eval for If<No, Then, Else> {
        type Output = Else;
    }

    #[test]
    fn if_supports_direct_eval_semantics() {
        assert_type_eq_all!(Evaluate<If<Yes, u8, u16>>, u8);
        assert_type_eq_all!(Evaluate<If<No, u8, u16>>, u16);
    }
}
