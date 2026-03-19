use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, TyFn};

use crate::core::traits::{
    Bind, Err, Monad, MonadError, MonadState, MonadTrans, MonadWriter, Ok, Pure,
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

pub trait RunEither {
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
    type Tell<Item> = EitherLift<E, M, <M as MonadWriter<W>>::Tell<Item>>;
}

impl<E, M, A> RunEither for EitherPure<E, M, A>
where
    M: Monad,
    Pure<M, Ok<A>>: Eval,
{
    type Output = Evaluate<Pure<M, Ok<A>>>;
}

impl<E, M, Reason> RunEither for EitherThrow<E, M, Reason>
where
    M: Monad,
    Pure<M, Err<Reason>>: Eval,
{
    type Output = Evaluate<Pure<M, Err<Reason>>>;
}

pub struct LEitherLiftCont<M>(pub PhantomData<M>);

impl<M, A> TyFn<A> for LEitherLiftCont<M>
where
    M: Monad,
    Pure<M, Ok<A>>: Eval,
{
    type Output = Pure<M, Ok<A>>;
}

impl<E, M, MA> RunEither for EitherLift<E, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LEitherLiftCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LEitherLiftCont<M>>>;
}

pub struct LEitherBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A> TyFn<Ok<A>> for LEitherBindCont<M, K>
where
    K: TyFn<A>,
    <K as TyFn<A>>::Output: RunEither,
{
    type Output = <<K as TyFn<A>>::Output as RunEither>::Output;
}

impl<M, K, Reason> TyFn<Err<Reason>> for LEitherBindCont<M, K>
where
    M: Monad,
    Pure<M, Err<Reason>>: Eval,
{
    type Output = Pure<M, Err<Reason>>;
}

impl<E, M, MA, K> RunEither for EitherBind<E, M, MA, K>
where
    M: Monad,
    MA: RunEither,
    Bind<M, <MA as RunEither>::Output, LEitherBindCont<M, K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as RunEither>::Output, LEitherBindCont<M, K>>>;
}

pub struct ERunEither<MA>(pub PhantomData<MA>);

impl<MA> Eval for ERunEither<MA>
where
    MA: Eval,
    Evaluate<MA>: RunEither,
{
    type Output = <Evaluate<MA> as RunEither>::Output;
}
