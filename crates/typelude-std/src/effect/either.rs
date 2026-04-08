use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, Op};

use super::{
    Bind, Err, Monad, MonadError, MonadReader, MonadState, MonadSuspend, MonadTrans, MonadWriter,
    Ok, Pair, Pure,
};

#[derive(Debug)]
pub struct EitherT<E, M>(pub PhantomData<(E, M)>);

#[derive(Debug)]
pub struct EitherPure<E, M, A>(pub PhantomData<(E, M, A)>);

#[derive(Debug)]
pub struct EitherBind<E, M, MA, K>(pub PhantomData<(E, M, MA, K)>);

#[derive(Debug)]
pub struct EitherThrow<E, M, Reason>(pub PhantomData<(E, M, Reason)>);

#[derive(Debug)]
pub struct EitherLift<E, M, MA>(pub PhantomData<(E, M, MA)>);

#[derive(Debug)]
pub struct EitherLocal<Env, E, M, F, MA>(pub PhantomData<(Env, E, M, F, MA)>);

#[derive(Debug)]
pub struct EitherListen<W, E, M, MA>(pub PhantomData<(W, E, M, MA)>);

#[derive(Debug)]
pub struct EitherCensor<W, E, M, F, MA>(pub PhantomData<(W, E, M, F, MA)>);

#[derive(Debug)]
pub struct EitherCatch<E, M, MA, H>(pub PhantomData<(E, M, MA, H)>);

impl<E, M, A> Eval for EitherPure<E, M, A> {
    type Output = Self;
}

impl<E, M, MA, K> Eval for EitherBind<E, M, MA, K> {
    type Output = Self;
}

impl<E, M, Reason> Eval for EitherThrow<E, M, Reason> {
    type Output = Self;
}

impl<E, M, MA> Eval for EitherLift<E, M, MA> {
    type Output = Self;
}

impl<Env, E, M, F, MA> Eval for EitherLocal<Env, E, M, F, MA> {
    type Output = Self;
}

impl<W, E, M, MA> Eval for EitherListen<W, E, M, MA> {
    type Output = Self;
}

impl<W, E, M, F, MA> Eval for EitherCensor<W, E, M, F, MA> {
    type Output = Self;
}

impl<E, M, MA, H> Eval for EitherCatch<E, M, MA, H> {
    type Output = Self;
}

#[doc(hidden)]
pub trait EitherRunner {
    type Output;
}

impl<E, M> Monad for EitherT<E, M> {
    type Pure<A> = EitherPure<E, M, A>;
    type Bind<MA, K> = EitherBind<E, M, MA, K>;
}

impl<E, M> MonadTrans<M> for EitherT<E, M>
where
    M: Monad,
{
    type Lift<MA> = EitherLift<E, M, MA>;
}

impl<E, M> MonadError<E> for EitherT<E, M>
where
    M: Monad,
{
    type Throw<Reason> = EitherThrow<E, M, Reason>;
    type Catch<MA, H> = EitherCatch<E, M, MA, H>;
}

impl<E, M, Env> MonadReader<Env> for EitherT<E, M>
where
    M: Monad + MonadReader<Env>,
{
    type Ask = EitherLift<E, M, <M as MonadReader<Env>>::Ask>;
    type Local<F, MA> = EitherLocal<Env, E, M, F, MA>;
}

impl<E, M, S> MonadState<S> for EitherT<E, M>
where
    M: Monad + MonadState<S>,
{
    type Get = EitherLift<E, M, <M as MonadState<S>>::Get>;
    type Put<NewState> = EitherLift<E, M, <M as MonadState<S>>::Put<NewState>>;
    type Modify<F> = EitherLift<E, M, <M as MonadState<S>>::Modify<F>>;
}

impl<E, M, W> MonadWriter<W> for EitherT<E, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Chunk> = EitherLift<E, M, <M as MonadWriter<W>>::Tell<Chunk>>;
    type Listen<MA> = EitherListen<W, E, M, MA>;
    type Censor<F, MA> = EitherCensor<W, E, M, F, MA>;
}

impl<E, M, Req> MonadSuspend<Req> for EitherT<E, M>
where
    M: Monad + MonadSuspend<Req>,
{
    type Suspend<Request> = EitherLift<E, M, <M as MonadSuspend<Req>>::Suspend<Request>>;
}

impl<E, M, A> EitherRunner for EitherPure<E, M, A>
where
    M: Monad,
    Pure<M, Ok<A>>: Eval,
{
    type Output = Evaluate<Pure<M, Ok<A>>>;
}

impl<E, M, Reason> EitherRunner for EitherThrow<E, M, Reason>
where
    M: Monad,
    Pure<M, Err<Reason>>: Eval,
{
    type Output = Evaluate<Pure<M, Err<Reason>>>;
}

pub struct LEitherLiftCont<M>(pub PhantomData<M>);

impl<M, A> Op<A> for LEitherLiftCont<M>
where
    M: Monad,
    Pure<M, Ok<A>>: Eval,
{
    type Output = Pure<M, Ok<A>>;
}

impl<E, M, MA> EitherRunner for EitherLift<E, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LEitherLiftCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LEitherLiftCont<M>>>;
}

pub struct LEitherBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A> Op<Ok<A>> for LEitherBindCont<M, K>
where
    K: Op<A>,
    <K as Op<A>>::Output: Eval,
    Evaluate<<K as Op<A>>::Output>: EitherRunner,
{
    type Output = <Evaluate<<K as Op<A>>::Output> as EitherRunner>::Output;
}

impl<M, K, Reason> Op<Err<Reason>> for LEitherBindCont<M, K>
where
    M: Monad,
    Pure<M, Err<Reason>>: Eval,
{
    type Output = Pure<M, Err<Reason>>;
}

impl<E, M, MA, K> EitherRunner for EitherBind<E, M, MA, K>
where
    M: Monad,
    MA: EitherRunner,
    Bind<M, <MA as EitherRunner>::Output, LEitherBindCont<M, K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as EitherRunner>::Output, LEitherBindCont<M, K>>>;
}

impl<Env, E, M, F, MA> EitherRunner for EitherLocal<Env, E, M, F, MA>
where
    M: Monad + MonadReader<Env>,
    MA: EitherRunner,
    <M as MonadReader<Env>>::Local<F, <MA as EitherRunner>::Output>: Eval,
{
    type Output = Evaluate<<M as MonadReader<Env>>::Local<F, <MA as EitherRunner>::Output>>;
}

pub struct LEitherListenCont<M>(pub PhantomData<M>);

impl<M, A, W> Op<Pair<Ok<A>, W>> for LEitherListenCont<M>
where
    M: Monad,
    Pure<M, Ok<Pair<A, W>>>: Eval,
{
    type Output = Pure<M, Ok<Pair<A, W>>>;
}

impl<M, Reason, W> Op<Pair<Err<Reason>, W>> for LEitherListenCont<M>
where
    M: Monad,
    Pure<M, Err<Reason>>: Eval,
{
    type Output = Pure<M, Err<Reason>>;
}

impl<W, E, M, MA> EitherRunner for EitherListen<W, E, M, MA>
where
    M: Monad + MonadWriter<W>,
    MA: EitherRunner,
    <M as MonadWriter<W>>::Listen<<MA as EitherRunner>::Output>: Eval,
    Bind<
        M,
        Evaluate<<M as MonadWriter<W>>::Listen<<MA as EitherRunner>::Output>>,
        LEitherListenCont<M>,
    >: Eval,
{
    type Output = Evaluate<
        Bind<
            M,
            Evaluate<<M as MonadWriter<W>>::Listen<<MA as EitherRunner>::Output>>,
            LEitherListenCont<M>,
        >,
    >;
}

impl<W, E, M, F, MA> EitherRunner for EitherCensor<W, E, M, F, MA>
where
    M: Monad + MonadWriter<W>,
    MA: EitherRunner,
    <M as MonadWriter<W>>::Censor<F, <MA as EitherRunner>::Output>: Eval,
{
    type Output = Evaluate<<M as MonadWriter<W>>::Censor<F, <MA as EitherRunner>::Output>>;
}

pub struct LEitherCatchCont<M, H>(pub PhantomData<(M, H)>);

impl<M, H, A> Op<Ok<A>> for LEitherCatchCont<M, H>
where
    M: Monad,
    Pure<M, Ok<A>>: Eval,
{
    type Output = Pure<M, Ok<A>>;
}

impl<M, H, Reason> Op<Err<Reason>> for LEitherCatchCont<M, H>
where
    H: Op<Reason>,
    <H as Op<Reason>>::Output: Eval,
    Evaluate<<H as Op<Reason>>::Output>: EitherRunner,
{
    type Output = <Evaluate<<H as Op<Reason>>::Output> as EitherRunner>::Output;
}

impl<E, M, MA, H> EitherRunner for EitherCatch<E, M, MA, H>
where
    M: Monad,
    MA: EitherRunner,
    Bind<M, <MA as EitherRunner>::Output, LEitherCatchCont<M, H>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as EitherRunner>::Output, LEitherCatchCont<M, H>>>;
}

#[derive(Debug)]
pub struct RunEither<MA>(pub PhantomData<MA>);

impl<MA> Eval for RunEither<MA>
where
    MA: Eval,
    Evaluate<MA>: EitherRunner,
{
    type Output = <Evaluate<MA> as EitherRunner>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{IdK, RunId, Unit};

    #[test]
    fn run_either_pure_wraps_ok() {
        type Result = Evaluate<RunId<RunEither<Pure<EitherT<Unit, IdK>, Unit>>>>;
        assert_type_eq_all!(Result, Ok<Unit>);
    }

    #[test]
    fn run_either_throw_wraps_err() {
        type Program = <EitherT<Unit, IdK> as MonadError<Unit>>::Throw<Unit>;
        type Result = Evaluate<RunId<RunEither<Program>>>;
        assert_type_eq_all!(Result, Err<Unit>);
    }

    #[test]
    fn run_either_bind_short_circuits_err() {
        type Program = Bind<
            EitherT<Unit, IdK>,
            <EitherT<Unit, IdK> as MonadError<Unit>>::Throw<Unit>,
            super::super::LConst<Pure<EitherT<Unit, IdK>, Unit>>,
        >;
        type Result = Evaluate<RunId<RunEither<Program>>>;
        assert_type_eq_all!(Result, Err<Unit>);
    }

    #[test]
    fn run_either_lift_promotes_inner_value_into_ok() {
        type Program = EitherLift<Unit, IdK, Pure<IdK, Unit>>;
        type Result = Evaluate<RunId<RunEither<Program>>>;
        assert_type_eq_all!(Result, Ok<Unit>);
    }

    #[test]
    fn run_either_catch_recovers_err() {
        type Program = <EitherT<Unit, IdK> as MonadError<Unit>>::Catch<
            <EitherT<Unit, IdK> as MonadError<Unit>>::Throw<Unit>,
            super::super::LConst<Pure<EitherT<Unit, IdK>, Unit>>,
        >;
        type Result = Evaluate<RunId<RunEither<Program>>>;
        assert_type_eq_all!(Result, Ok<Unit>);
    }
}
