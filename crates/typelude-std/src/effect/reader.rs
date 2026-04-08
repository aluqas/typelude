use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, Op};

use super::{
    Bind, Monad, MonadError, MonadReader, MonadState, MonadSuspend, MonadTrans, MonadWriter, Pure,
};

#[derive(Debug)]
pub struct ReaderT<R, M>(pub PhantomData<(R, M)>);

#[derive(Debug)]
pub struct ReaderPure<R, M, A>(pub PhantomData<(R, M, A)>);

#[derive(Debug)]
pub struct ReaderBind<R, M, MA, K>(pub PhantomData<(R, M, MA, K)>);

#[derive(Debug)]
pub struct ReaderAsk<R, M>(pub PhantomData<(R, M)>);

#[derive(Debug)]
pub struct ReaderLocal<R, M, F, MA>(pub PhantomData<(R, M, F, MA)>);

#[derive(Debug)]
pub struct ReaderLift<R, M, MA>(pub PhantomData<(R, M, MA)>);

#[derive(Debug)]
pub struct ReaderListen<W, R, M, MA>(pub PhantomData<(W, R, M, MA)>);

#[derive(Debug)]
pub struct ReaderCensor<W, R, M, F, MA>(pub PhantomData<(W, R, M, F, MA)>);

#[derive(Debug)]
pub struct ReaderCatch<E, R, M, MA, H>(pub PhantomData<(E, R, M, MA, H)>);

impl<R, M, A> Eval for ReaderPure<R, M, A> {
    type Output = Self;
}

impl<R, M, MA, K> Eval for ReaderBind<R, M, MA, K> {
    type Output = Self;
}

impl<R, M> Eval for ReaderAsk<R, M> {
    type Output = Self;
}

impl<R, M, F, MA> Eval for ReaderLocal<R, M, F, MA> {
    type Output = Self;
}

impl<R, M, MA> Eval for ReaderLift<R, M, MA> {
    type Output = Self;
}

impl<W, R, M, MA> Eval for ReaderListen<W, R, M, MA> {
    type Output = Self;
}

impl<W, R, M, F, MA> Eval for ReaderCensor<W, R, M, F, MA> {
    type Output = Self;
}

impl<E, R, M, MA, H> Eval for ReaderCatch<E, R, M, MA, H> {
    type Output = Self;
}

#[doc(hidden)]
pub trait ReaderRunner<Env> {
    type Output;
}

impl<R, M> Monad for ReaderT<R, M> {
    type Pure<A> = ReaderPure<R, M, A>;
    type Bind<MA, K> = ReaderBind<R, M, MA, K>;
}

impl<R, M> MonadTrans<M> for ReaderT<R, M>
where
    M: Monad,
{
    type Lift<MA> = ReaderLift<R, M, MA>;
}

impl<R, M> MonadReader<R> for ReaderT<R, M>
where
    M: Monad,
{
    type Ask = ReaderAsk<R, M>;
    type Local<F, MA> = ReaderLocal<R, M, F, MA>;
}

impl<R, M, S> MonadState<S> for ReaderT<R, M>
where
    M: Monad + MonadState<S>,
{
    type Get = ReaderLift<R, M, <M as MonadState<S>>::Get>;
    type Put<NewState> = ReaderLift<R, M, <M as MonadState<S>>::Put<NewState>>;
    type Modify<F> = ReaderLift<R, M, <M as MonadState<S>>::Modify<F>>;
}

impl<R, M, W> MonadWriter<W> for ReaderT<R, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Chunk> = ReaderLift<R, M, <M as MonadWriter<W>>::Tell<Chunk>>;
    type Listen<MA> = ReaderListen<W, R, M, MA>;
    type Censor<F, MA> = ReaderCensor<W, R, M, F, MA>;
}

impl<R, M, E> MonadError<E> for ReaderT<R, M>
where
    M: Monad + MonadError<E>,
{
    type Throw<Reason> = ReaderLift<R, M, <M as MonadError<E>>::Throw<Reason>>;
    type Catch<MA, H> = ReaderCatch<E, R, M, MA, H>;
}

impl<R, M, Req> MonadSuspend<Req> for ReaderT<R, M>
where
    M: Monad + MonadSuspend<Req>,
{
    type Suspend<Request> = ReaderLift<R, M, <M as MonadSuspend<Req>>::Suspend<Request>>;
}

impl<Env, R, M, A> ReaderRunner<Env> for ReaderPure<R, M, A>
where
    M: Monad,
    Pure<M, A>: Eval,
{
    type Output = Evaluate<Pure<M, A>>;
}

impl<Env, R, M> ReaderRunner<Env> for ReaderAsk<R, M>
where
    M: Monad,
    Pure<M, Env>: Eval,
{
    type Output = Evaluate<Pure<M, Env>>;
}

impl<Env, R, M, MA> ReaderRunner<Env> for ReaderLift<R, M, MA>
where
    MA: Eval,
{
    type Output = Evaluate<MA>;
}

pub struct LReaderBindCont<Env, K>(pub PhantomData<(Env, K)>);

impl<Env, K, A> Op<A> for LReaderBindCont<Env, K>
where
    K: Op<A>,
    <K as Op<A>>::Output: Eval,
    Evaluate<<K as Op<A>>::Output>: ReaderRunner<Env>,
{
    type Output = <Evaluate<<K as Op<A>>::Output> as ReaderRunner<Env>>::Output;
}

impl<Env, R, M, MA, K> ReaderRunner<Env> for ReaderBind<R, M, MA, K>
where
    M: Monad,
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Env>,
    Bind<M, <Evaluate<MA> as ReaderRunner<Env>>::Output, LReaderBindCont<Env, K>>: Eval,
{
    type Output =
        Evaluate<Bind<M, <Evaluate<MA> as ReaderRunner<Env>>::Output, LReaderBindCont<Env, K>>>;
}

impl<Env, R, M, F, MA> ReaderRunner<Env> for ReaderLocal<R, M, F, MA>
where
    F: Op<Env>,
    <F as Op<Env>>::Output: Eval,
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Evaluate<<F as Op<Env>>::Output>>,
{
    type Output = <Evaluate<MA> as ReaderRunner<Evaluate<<F as Op<Env>>::Output>>>::Output;
}

impl<Env, W, R, M, MA> ReaderRunner<Env> for ReaderListen<W, R, M, MA>
where
    M: Monad + MonadWriter<W>,
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Env>,
    <M as MonadWriter<W>>::Listen<<Evaluate<MA> as ReaderRunner<Env>>::Output>: Eval,
{
    type Output =
        Evaluate<<M as MonadWriter<W>>::Listen<<Evaluate<MA> as ReaderRunner<Env>>::Output>>;
}

impl<Env, W, R, M, F, MA> ReaderRunner<Env> for ReaderCensor<W, R, M, F, MA>
where
    M: Monad + MonadWriter<W>,
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Env>,
    <M as MonadWriter<W>>::Censor<F, <Evaluate<MA> as ReaderRunner<Env>>::Output>: Eval,
{
    type Output =
        Evaluate<<M as MonadWriter<W>>::Censor<F, <Evaluate<MA> as ReaderRunner<Env>>::Output>>;
}

pub struct LReaderCatchCont<Env, H>(pub PhantomData<(Env, H)>);

impl<Env, H, Reason> Op<Reason> for LReaderCatchCont<Env, H>
where
    H: Op<Reason>,
    <H as Op<Reason>>::Output: Eval,
    Evaluate<<H as Op<Reason>>::Output>: ReaderRunner<Env>,
{
    type Output = <Evaluate<<H as Op<Reason>>::Output> as ReaderRunner<Env>>::Output;
}

impl<Env, E, R, M, MA, H> ReaderRunner<Env> for ReaderCatch<E, R, M, MA, H>
where
    M: Monad + MonadError<E>,
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Env>,
    <M as MonadError<E>>::Catch<
        <Evaluate<MA> as ReaderRunner<Env>>::Output,
        LReaderCatchCont<Env, H>,
    >: Eval,
{
    type Output = Evaluate<
        <M as MonadError<E>>::Catch<
            <Evaluate<MA> as ReaderRunner<Env>>::Output,
            LReaderCatchCont<Env, H>,
        >,
    >;
}

#[derive(Debug)]
pub struct RunReader<Env, MA>(pub PhantomData<(Env, MA)>);

impl<Env, MA> Eval for RunReader<Env, MA>
where
    MA: Eval,
    Evaluate<MA>: ReaderRunner<Env>,
{
    type Output = <Evaluate<MA> as ReaderRunner<Env>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{IdK, RunId, Unit};

    struct NewEnv;
    impl crate::core::Value for NewEnv {}

    #[test]
    fn run_reader_pure_ignores_environment() {
        type Program = Pure<ReaderT<Unit, IdK>, Unit>;
        type Result = Evaluate<RunId<RunReader<NewEnv, Program>>>;
        assert_type_eq_all!(Result, Unit);
    }

    #[test]
    fn run_reader_ask_returns_current_environment() {
        type Program = ReaderAsk<Unit, IdK>;
        type Result = Evaluate<RunId<RunReader<NewEnv, Program>>>;
        assert_type_eq_all!(Result, NewEnv);
    }

    #[test]
    fn run_reader_local_replaces_environment_for_subcomputation() {
        type Program = ReaderLocal<Unit, IdK, super::super::LConst<NewEnv>, ReaderAsk<Unit, IdK>>;
        type Result = Evaluate<RunId<RunReader<Unit, Program>>>;
        assert_type_eq_all!(Result, NewEnv);
    }
}
