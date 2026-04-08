//! Type-level effect system primitives and transformer stack building blocks.
//!
//! The public entry point is the `effect` module itself: choose a base monad
//! such as [`IdK`], layer transformers like [`EitherT`] or [`StateT`], then
//! run the stack explicitly with the corresponding `Run*` wrappers.
//!
//! ```rust
//! use core::marker::PhantomData;
//!
//! use typelude_std::{
//!     Evaluate,
//!     effect::{
//!         EitherT, IdK, Ok, Pair, Pure, ReaderT, RunEither, RunId, RunReader, RunState, StateT,
//!         Unit,
//!     },
//! };
//!
//! type Stack = ReaderT<Unit, StateT<Unit, EitherT<Unit, IdK>>>;
//! type Program = Pure<Stack, Unit>;
//! type Result = Evaluate<RunId<RunEither<RunState<Unit, RunReader<Unit, Program>>>>>;
//!
//! let _: PhantomData<Ok<Pair<Unit, Unit>>> = PhantomData::<Result>;
//! ```

pub mod combinator;
pub mod core;
pub mod effect;
pub mod either;
pub mod id;
pub mod reader;
pub mod state;
pub mod suspend;
pub mod writer;

pub use core::{
    Append, Bind, Empty, LConst, Monad, MonadError, MonadReader, MonadState, MonadSuspend,
    MonadTrans, MonadWriter, Pure,
};

pub use combinator::{Ap, Join, Map, Then};
pub use effect::{Done, Err, Ok, Pair, Unit, Yielded};
pub use either::{EitherBind, EitherLift, EitherPure, EitherT, EitherThrow, RunEither};
pub use id::{Id, IdBind, IdK, RunId, UnwrapId};
pub use reader::{
    ReaderAsk, ReaderBind, ReaderCatch, ReaderCensor, ReaderLift, ReaderListen, ReaderLocal,
    ReaderPure, ReaderT, RunReader,
};
pub use state::{
    RunState, StateBind, StateGet, StateLift, StateModify, StatePure, StatePut, StateT,
};
pub use suspend::{
    RunSuspend, SuspendBind, SuspendCatch, SuspendCensor, SuspendLift, SuspendListen,
    SuspendLocal, SuspendPure, SuspendT, SuspendYield,
};
pub use writer::{RunWriter, WriterBind, WriterLift, WriterPure, WriterT, WriterTell};

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{
        core::Evaluate,
        effect::{
            EitherT, IdK, Ok, Pair, Pure, ReaderT, RunEither, RunId, RunReader, RunState, StateT,
            Unit,
        },
    };

    #[test]
    fn effect_stack_usage_example_runs_reader_state_either_over_id() {
        type Stack = ReaderT<Unit, StateT<Unit, EitherT<Unit, IdK>>>;
        type Program = Pure<Stack, Unit>;
        type Result = Evaluate<RunId<RunEither<RunState<Unit, RunReader<Unit, Program>>>>>;

        assert_type_eq_all!(Result, Ok<Pair<Unit, Unit>>);
    }
}
