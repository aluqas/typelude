use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, TyFn};

use crate::composed::core::traits::{
    Bind, Monad, MonadState, MonadTrans, MonadWriter, Pair, Pure, Unit,
};

#[derive(Debug)]
pub struct StateT<S, M>(pub PhantomData<(S, M)>);

#[derive(Debug)]
pub struct StatePure<S, M, A>(pub PhantomData<(S, M, A)>);

#[derive(Debug)]
pub struct StateBind<S, M, MA, K>(pub PhantomData<(S, M, MA, K)>);

#[derive(Debug)]
pub struct StateGet<S, M>(pub PhantomData<(S, M)>);

#[derive(Debug)]
pub struct StatePut<S, M, NewState>(pub PhantomData<(S, M, NewState)>);

#[derive(Debug)]
pub struct StateModify<S, M, F>(pub PhantomData<(S, M, F)>);

#[derive(Debug)]
pub struct StateLift<S, M, MA>(pub PhantomData<(S, M, MA)>);

impl<S, M, A> Eval for StatePure<S, M, A> {
    type Output = Self;
}

impl<S, M, MA, K> Eval for StateBind<S, M, MA, K> {
    type Output = Self;
}

impl<S, M> Eval for StateGet<S, M> {
    type Output = Self;
}

impl<S, M, NewState> Eval for StatePut<S, M, NewState> {
    type Output = Self;
}

impl<S, M, F> Eval for StateModify<S, M, F> {
    type Output = Self;
}

impl<S, M, MA> Eval for StateLift<S, M, MA> {
    type Output = Self;
}

pub trait RunState<Init> {
    type Output;
}

impl<S, M> Monad for StateT<S, M> {
    type Pure<A> = StatePure<S, M, A>;
    type Bind<MA, K> = StateBind<S, M, MA, K>;
}

impl<S, M> MonadTrans<M> for StateT<S, M>
where
    M: Monad,
{
    type Lift<MA> = StateLift<S, M, MA>;
}

impl<S, M> MonadState<S> for StateT<S, M>
where
    M: Monad,
{
    type Get = StateGet<S, M>;
    type Put<NewState> = StatePut<S, M, NewState>;
    type Modify<F> = StateModify<S, M, F>;
}

impl<S, M, W> MonadWriter<W> for StateT<S, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Item> = StateLift<S, M, <M as MonadWriter<W>>::Tell<Item>>;
}

impl<Init, S, M, A> RunState<Init> for StatePure<S, M, A>
where
    M: Monad,
    Pure<M, Pair<A, Init>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<A, Init>>>;
}

impl<Init, S, M> RunState<Init> for StateGet<S, M>
where
    M: Monad,
    Pure<M, Pair<Init, Init>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Init, Init>>>;
}

impl<Init, S, M, NewState> RunState<Init> for StatePut<S, M, NewState>
where
    M: Monad,
    Pure<M, Pair<Unit, NewState>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, NewState>>>;
}

impl<Init, S, M, F> RunState<Init> for StateModify<S, M, F>
where
    M: Monad,
    F: TyFn<Init>,
    Pure<M, Pair<Unit, <F as TyFn<Init>>::Output>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, <F as TyFn<Init>>::Output>>>;
}

pub struct LStateLiftCont<M, Init>(pub PhantomData<(M, Init)>);

impl<M, Init, A> TyFn<A> for LStateLiftCont<M, Init>
where
    M: Monad,
    Pure<M, Pair<A, Init>>: Eval,
{
    type Output = Pure<M, Pair<A, Init>>;
}

impl<Init, S, M, MA> RunState<Init> for StateLift<S, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LStateLiftCont<M, Init>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LStateLiftCont<M, Init>>>;
}

pub struct LStateBindCont<K>(pub PhantomData<K>);

impl<A, NextState, K> TyFn<Pair<A, NextState>> for LStateBindCont<K>
where
    K: TyFn<A>,
    <K as TyFn<A>>::Output: RunState<NextState>,
{
    type Output = <<K as TyFn<A>>::Output as RunState<NextState>>::Output;
}

impl<Init, S, M, MA, K> RunState<Init> for StateBind<S, M, MA, K>
where
    MA: RunState<Init>,
    M: Monad,
    Bind<M, <MA as RunState<Init>>::Output, LStateBindCont<K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as RunState<Init>>::Output, LStateBindCont<K>>>;
}

pub struct ERunState<Init, MA>(pub PhantomData<(Init, MA)>);

impl<Init, MA> Eval for ERunState<Init, MA>
where
    MA: Eval,
    Evaluate<MA>: RunState<Init>,
{
    type Output = <Evaluate<MA> as RunState<Init>>::Output;
}
