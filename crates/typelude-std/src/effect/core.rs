use core::marker::PhantomData;

use crate::core::Op;

/// Type-level monad interface.
///
/// `Pure<F, A>` and `Bind<F, MA, K>` are the canonical aliases for building
/// monadic expressions from a monad kind `F`.
///
/// ```rust
/// use core::marker::PhantomData;
///
/// use typelude_std::{
///     Evaluate, Op,
///     effect::{Bind, IdK, Pair, Pure, RunId, Unit},
/// };
///
/// struct Duplicate;
///
/// impl Op<Unit> for Duplicate {
///     type Output = Pure<IdK, Pair<Unit, Unit>>;
/// }
///
/// type Program = Bind<IdK, Pure<IdK, Unit>, Duplicate>;
/// type Result = Evaluate<RunId<Program>>;
///
/// let _: PhantomData<Pair<Unit, Unit>> = PhantomData::<Result>;
/// ```
pub trait Monad {
    type Pure<A>;
    type Bind<MA, K>;
}

pub type Pure<F, A> = <F as Monad>::Pure<A>;
pub type Bind<F, MA, K> = <F as Monad>::Bind<MA, K>;

pub trait MonadTrans<Inner>: Monad {
    type Lift<MA>;
}

pub trait MonadReader<R>: Monad {
    type Ask;
    type Local<F, MA>;
}

pub trait MonadState<S>: Monad {
    type Get;
    type Put<NewState>;
    type Modify<F>;
}

pub trait MonadWriter<W>: Monad {
    type Tell<Chunk>;
    type Listen<MA>;
    type Censor<F, MA>;
}

pub trait MonadError<E>: Monad {
    type Throw<Reason>;
    type Catch<MA, H>;
}

pub trait MonadSuspend<R>: Monad {
    type Suspend<Request>;
}

pub trait Empty {
    type Output;
}

pub trait Append<Rhs> {
    type Output;
}

pub struct LConst<T>(pub PhantomData<T>);

impl<A, T> Op<A> for LConst<T> {
    type Output = T;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::{
        core::Evaluate,
        effect::{IdK, Pair, RunId, Unit},
    };

    struct Duplicate;

    impl Op<Unit> for Duplicate {
        type Output = Pure<IdK, Pair<Unit, Unit>>;
    }

    #[test]
    fn monad_usage_example_builds_id_program() {
        type Program = Bind<IdK, Pure<IdK, Unit>, Duplicate>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn lconst_can_be_used_as_bind_continuation() {
        type Program = Bind<IdK, Pure<IdK, Unit>, LConst<Pure<IdK, Pair<Unit, Unit>>>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }
}
