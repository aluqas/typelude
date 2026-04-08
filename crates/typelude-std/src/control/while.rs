//! While-loop expression AST.

use core::marker::PhantomData;

/// While-loop expression AST.
///
/// `Pred` and `Step` are expected to be first-class operators when used by a
/// concrete implementation.
///
/// Direct `Eval` implementations are still allowed for custom control-flow
/// semantics that need lazy branching or domain-specific execution rules.
///
/// # Examples
///
/// ```ignore
/// use static_assertions::assert_type_eq_all;
/// use typelude_std::{Eval, Evaluate, While};
///
/// struct Halt;
/// struct Step;
///
/// impl<State> Eval for While<Halt, Step, State> {
///     type Output = State;
/// }
///
/// assert_type_eq_all!(Evaluate<While<Halt, Step, u8>>, u8);
/// ```
pub struct While<Pred, Step, State>(pub PhantomData<(Pred, Step, State)>);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{Eval, Evaluate, While};

    struct Halt;
    struct Step;

    impl<State> Eval for While<Halt, Step, State> {
        type Output = State;
    }

    #[test]
    fn while_supports_direct_eval_semantics() {
        assert_type_eq_all!(Evaluate<While<Halt, Step, u8>>, u8);
    }
}
