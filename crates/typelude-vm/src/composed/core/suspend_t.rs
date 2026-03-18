use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, TyFn};

use crate::composed::core::traits::{
    Bind, Done, Monad, MonadError, MonadState, MonadSuspend, MonadTrans, MonadWriter, Pure, Yielded,
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

pub trait RunSuspend {
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

impl<R, M, W> MonadWriter<W> for SuspendT<R, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Item> = SuspendLift<R, M, <M as MonadWriter<W>>::Tell<Item>>;
}

impl<R, M, E> MonadError<E> for SuspendT<R, M>
where
    M: Monad + MonadError<E>,
{
    type Throw<Reason> = SuspendLift<R, M, <M as MonadError<E>>::Throw<Reason>>;
}

impl<R, M, A> RunSuspend for SuspendPure<R, M, A>
where
    M: Monad,
    Pure<M, Done<A>>: Eval,
{
    type Output = Evaluate<Pure<M, Done<A>>>;
}

impl<R, M, Request> RunSuspend for SuspendYield<R, M, Request>
where
    M: Monad,
    Pure<M, Yielded<Request>>: Eval,
{
    type Output = Evaluate<Pure<M, Yielded<Request>>>;
}

pub struct LSuspendLiftCont<M>(pub PhantomData<M>);

impl<M, A> TyFn<A> for LSuspendLiftCont<M>
where
    M: Monad,
    Pure<M, Done<A>>: Eval,
{
    type Output = Pure<M, Done<A>>;
}

impl<R, M, MA> RunSuspend for SuspendLift<R, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LSuspendLiftCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LSuspendLiftCont<M>>>;
}

pub struct LSuspendBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A> TyFn<Done<A>> for LSuspendBindCont<M, K>
where
    K: TyFn<A>,
    <K as TyFn<A>>::Output: RunSuspend,
{
    type Output = <<K as TyFn<A>>::Output as RunSuspend>::Output;
}

impl<M, K, Request> TyFn<Yielded<Request>> for LSuspendBindCont<M, K>
where
    M: Monad,
    Pure<M, Yielded<Request>>: Eval,
{
    type Output = Pure<M, Yielded<Request>>;
}

impl<R, M, MA, K> RunSuspend for SuspendBind<R, M, MA, K>
where
    M: Monad,
    MA: RunSuspend,
    Bind<M, <MA as RunSuspend>::Output, LSuspendBindCont<M, K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as RunSuspend>::Output, LSuspendBindCont<M, K>>>;
}

pub struct ERunSuspend<MA>(pub PhantomData<MA>);

impl<MA> Eval for ERunSuspend<MA>
where
    MA: Eval,
    Evaluate<MA>: RunSuspend,
{
    type Output = <Evaluate<MA> as RunSuspend>::Output;
}
