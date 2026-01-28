//! **State Monad Primitives**
//!
//! Re-exports and utility functions for the State Monad.
//! Using `Apply` trait for reductions.

use std::marker::PhantomData;

pub use typelude_std::lambda::monads::state::{
    LBindGet, LBindPut, LBindState, LGet, LPut, LReturn, LState, Unit,
};
pub use typelude_std::lambda::traits::LBind;
use typelude_core::Apply;

use typelude_core::{Eval, Evaluate};

// =============================================================================
// LModify
// =============================================================================

/// Monadic Modify: `modify f = \s -> ((), f s)`
pub struct LModify<F>(PhantomData<F>);

impl<F> Eval for LModify<F> {
    type Output = LModify<F>;
}

// LModify<F> S -> ((), F S)
impl<F, S> Apply<S> for LModify<F>
where
    F: Eval + Apply<S>,
    S: Eval,
    <F as Apply<S>>::Output: Eval,
{
    // Returns (Unit, NewState)
    type Output = typelude_std::lambda::church::LPair2<Unit, Evaluate<<F as Apply<S>>::Output>>;
}

// Wrapper to treat LModify<F> as LState<LModify<F>> if needed by type system?
// In typelude, "State Actions" are usually just the Eval-driven `S -> (A, S)`.
// But `LState<T>` wrapper exists to tag them.
// Let's rely on the Eval signature.
// If we need explicit `LState` wrapping, we can add `type ActionModify<F> = LState<LModify<F>>`.

pub type ActionModify<F> = LState<LModify<F>>;
