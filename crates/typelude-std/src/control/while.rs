//! While-loop expression AST.

use core::marker::PhantomData;

/// While-loop expression AST.
///
/// `Pred` and `Step` are expected to be first-class operators when used by a
/// concrete implementation.
pub struct While<Pred, Step, State>(pub PhantomData<(Pred, Step, State)>);
