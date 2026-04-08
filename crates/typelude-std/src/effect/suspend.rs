use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, Op};

use super::{
    Bind, Done, Monad, MonadError, MonadReader, MonadState, MonadSuspend, MonadTrans, MonadWriter,
    Pair, Pure, Yielded,
};

#[derive(Debug)]
pub struct SuspendT<R, M>(pub PhantomData<(R, M)>);

#[derive(Debug)]
pub struct SuspendPure<R, M, A>(pub PhantomData<(R, M, A)>);

#[derive(Debug)]
pub struct SuspendBind<R, M, MA, K>(pub PhantomData<(R, M, MA, K)>);

#[derive(Debug)]
pub struct SuspendYield<R, M, Request>(pub PhantomData<(R, M, Request)>);

#[derive(Debug)]
pub struct SuspendLift<R, M, MA>(pub PhantomData<(R, M, MA)>);

#[derive(Debug)]
pub struct SuspendLocal<Env, R, M, F, MA>(pub PhantomData<(Env, R, M, F, MA)>);

#[derive(Debug)]
pub struct SuspendListen<W, R, M, MA>(pub PhantomData<(W, R, M, MA)>);

#[derive(Debug)]
pub struct SuspendCensor<W, R, M, F, MA>(pub PhantomData<(W, R, M, F, MA)>);

#[derive(Debug)]
pub struct SuspendCatch<E, R, M, MA, H>(pub PhantomData<(E, R, M, MA, H)>);

impl<R, M, A> Eval for SuspendPure<R, M, A> {
    type Output = Self;
}

impl<R, M, MA, K> Eval for SuspendBind<R, M, MA, K> {
    type Output = Self;
}

impl<R, M, Request> Eval for SuspendYield<R, M, Request> {
    type Output = Self;
}

impl<R, M, MA> Eval for SuspendLift<R, M, MA> {
    type Output = Self;
}

impl<Env, R, M, F, MA> Eval for SuspendLocal<Env, R, M, F, MA> {
    type Output = Self;
}

impl<W, R, M, MA> Eval for SuspendListen<W, R, M, MA> {
    type Output = Self;
}

impl<W, R, M, F, MA> Eval for SuspendCensor<W, R, M, F, MA> {
    type Output = Self;
}

impl<E, R, M, MA, H> Eval for SuspendCatch<E, R, M, MA, H> {
    type Output = Self;
}

#[doc(hidden)]
pub trait SuspendRunner {
    type Output;
}

impl<R, M> Monad for SuspendT<R, M> {
    type Pure<A> = SuspendPure<R, M, A>;
    type Bind<MA, K> = SuspendBind<R, M, MA, K>;
}

impl<R, M> MonadTrans<M> for SuspendT<R, M>
where
    M: Monad,
{
    type Lift<MA> = SuspendLift<R, M, MA>;
}

impl<R, M> MonadSuspend<R> for SuspendT<R, M>
where
    M: Monad,
{
    type Suspend<Request> = SuspendYield<R, M, Request>;
}

impl<R, M, S> MonadState<S> for SuspendT<R, M>
where
    M: Monad + MonadState<S>,
{
    type Get = SuspendLift<R, M, <M as MonadState<S>>::Get>;
    type Put<NewState> = SuspendLift<R, M, <M as MonadState<S>>::Put<NewState>>;
    type Modify<F> = SuspendLift<R, M, <M as MonadState<S>>::Modify<F>>;
}

impl<R, M, Env> MonadReader<Env> for SuspendT<R, M>
where
    M: Monad + MonadReader<Env>,
{
    type Ask = SuspendLift<R, M, <M as MonadReader<Env>>::Ask>;
    type Local<F, MA> = SuspendLocal<Env, R, M, F, MA>;
}

impl<R, M, W> MonadWriter<W> for SuspendT<R, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Chunk> = SuspendLift<R, M, <M as MonadWriter<W>>::Tell<Chunk>>;
    type Listen<MA> = SuspendListen<W, R, M, MA>;
    type Censor<F, MA> = SuspendCensor<W, R, M, F, MA>;
}

impl<R, M, E> MonadError<E> for SuspendT<R, M>
where
    M: Monad + MonadError<E>,
{
    type Throw<Reason> = SuspendLift<R, M, <M as MonadError<E>>::Throw<Reason>>;
    type Catch<MA, H> = SuspendCatch<E, R, M, MA, H>;
}

impl<R, M, A> SuspendRunner for SuspendPure<R, M, A>
where
    M: Monad,
    Pure<M, Done<A>>: Eval,
{
    type Output = Evaluate<Pure<M, Done<A>>>;
}

impl<R, M, Request> SuspendRunner for SuspendYield<R, M, Request>
where
    M: Monad,
    Pure<M, Yielded<Request>>: Eval,
{
    type Output = Evaluate<Pure<M, Yielded<Request>>>;
}

pub struct LSuspendLiftCont<M>(pub PhantomData<M>);

impl<M, A> Op<A> for LSuspendLiftCont<M>
where
    M: Monad,
    Pure<M, Done<A>>: Eval,
{
    type Output = Pure<M, Done<A>>;
}

impl<R, M, MA> SuspendRunner for SuspendLift<R, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LSuspendLiftCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LSuspendLiftCont<M>>>;
}

pub struct LSuspendBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A> Op<Done<A>> for LSuspendBindCont<M, K>
where
    K: Op<A>,
    <K as Op<A>>::Output: Eval,
    Evaluate<<K as Op<A>>::Output>: SuspendRunner,
{
    type Output = <Evaluate<<K as Op<A>>::Output> as SuspendRunner>::Output;
}

impl<M, K, Request> Op<Yielded<Request>> for LSuspendBindCont<M, K>
where
    M: Monad,
    Pure<M, Yielded<Request>>: Eval,
{
    type Output = Pure<M, Yielded<Request>>;
}

impl<R, M, MA, K> SuspendRunner for SuspendBind<R, M, MA, K>
where
    M: Monad,
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
    Bind<M, <Evaluate<MA> as SuspendRunner>::Output, LSuspendBindCont<M, K>>: Eval,
{
    type Output =
        Evaluate<Bind<M, <Evaluate<MA> as SuspendRunner>::Output, LSuspendBindCont<M, K>>>;
}

impl<Env, R, M, F, MA> SuspendRunner for SuspendLocal<Env, R, M, F, MA>
where
    M: Monad + MonadReader<Env>,
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
    <M as MonadReader<Env>>::Local<F, <Evaluate<MA> as SuspendRunner>::Output>: Eval,
{
    type Output =
        Evaluate<<M as MonadReader<Env>>::Local<F, <Evaluate<MA> as SuspendRunner>::Output>>;
}

pub struct LSuspendListenCont<M>(pub PhantomData<M>);

impl<M, A, W> Op<Pair<Done<A>, W>> for LSuspendListenCont<M>
where
    M: Monad,
    Pure<M, Done<Pair<A, W>>>: Eval,
{
    type Output = Pure<M, Done<Pair<A, W>>>;
}

impl<M, Request, W> Op<Pair<Yielded<Request>, W>> for LSuspendListenCont<M>
where
    M: Monad,
    Pure<M, Yielded<Request>>: Eval,
{
    type Output = Pure<M, Yielded<Request>>;
}

impl<W, R, M, MA> SuspendRunner for SuspendListen<W, R, M, MA>
where
    M: Monad + MonadWriter<W>,
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
    <M as MonadWriter<W>>::Listen<<Evaluate<MA> as SuspendRunner>::Output>: Eval,
    Bind<
        M,
        Evaluate<<M as MonadWriter<W>>::Listen<<Evaluate<MA> as SuspendRunner>::Output>>,
        LSuspendListenCont<M>,
    >: Eval,
{
    type Output = Evaluate<
        Bind<
            M,
            Evaluate<<M as MonadWriter<W>>::Listen<<Evaluate<MA> as SuspendRunner>::Output>>,
            LSuspendListenCont<M>,
        >,
    >;
}

impl<W, R, M, F, MA> SuspendRunner for SuspendCensor<W, R, M, F, MA>
where
    M: Monad + MonadWriter<W>,
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
    <M as MonadWriter<W>>::Censor<F, <Evaluate<MA> as SuspendRunner>::Output>: Eval,
{
    type Output =
        Evaluate<<M as MonadWriter<W>>::Censor<F, <Evaluate<MA> as SuspendRunner>::Output>>;
}

pub struct LSuspendCatchCont<H>(pub PhantomData<H>);

impl<H, Reason> Op<Reason> for LSuspendCatchCont<H>
where
    H: Op<Reason>,
    <H as Op<Reason>>::Output: Eval,
    Evaluate<<H as Op<Reason>>::Output>: SuspendRunner,
{
    type Output = <Evaluate<<H as Op<Reason>>::Output> as SuspendRunner>::Output;
}

impl<E, R, M, MA, H> SuspendRunner for SuspendCatch<E, R, M, MA, H>
where
    M: Monad + MonadError<E>,
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
    <M as MonadError<E>>::Catch<<Evaluate<MA> as SuspendRunner>::Output, LSuspendCatchCont<H>>:
        Eval,
{
    type Output = Evaluate<
        <M as MonadError<E>>::Catch<<Evaluate<MA> as SuspendRunner>::Output, LSuspendCatchCont<H>>,
    >;
}

#[derive(Debug)]
pub struct RunSuspend<MA>(pub PhantomData<MA>);

impl<MA> Eval for RunSuspend<MA>
where
    MA: Eval,
    Evaluate<MA>: SuspendRunner,
{
    type Output = <Evaluate<MA> as SuspendRunner>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{IdK, RunId, Unit};

    #[test]
    fn run_suspend_pure_returns_done() {
        type Result = Evaluate<RunId<RunSuspend<Pure<SuspendT<Unit, IdK>, Unit>>>>;
        assert_type_eq_all!(Result, Done<Unit>);
    }

    #[test]
    fn run_suspend_yield_returns_yielded() {
        type Program = <SuspendT<Unit, IdK> as MonadSuspend<Unit>>::Suspend<Unit>;
        type Result = Evaluate<RunId<RunSuspend<Program>>>;
        assert_type_eq_all!(Result, Yielded<Unit>);
    }

    #[test]
    fn run_suspend_bind_short_circuits_on_yield() {
        type Program = Bind<
            SuspendT<Unit, IdK>,
            <SuspendT<Unit, IdK> as MonadSuspend<Unit>>::Suspend<Unit>,
            super::super::LConst<Pure<SuspendT<Unit, IdK>, Unit>>,
        >;
        type Result = Evaluate<RunId<RunSuspend<Program>>>;
        assert_type_eq_all!(Result, Yielded<Unit>);
    }
}
