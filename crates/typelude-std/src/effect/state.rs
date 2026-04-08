use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, Op},
    effect::{
        Bind, Monad, MonadError, MonadReader, MonadState, MonadSuspend, MonadTrans, MonadWriter,
        Pair, Pure, Unit,
    },
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

#[derive(Debug)]
pub struct StateLocal<Env, S, M, F, MA>(pub PhantomData<(Env, S, M, F, MA)>);

#[derive(Debug)]
pub struct StateListen<W, S, M, MA>(pub PhantomData<(W, S, M, MA)>);

#[derive(Debug)]
pub struct StateCensor<W, S, M, F, MA>(pub PhantomData<(W, S, M, F, MA)>);

#[derive(Debug)]
pub struct StateCatch<E, S, M, MA, H>(pub PhantomData<(E, S, M, MA, H)>);

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

impl<Env, S, M, F, MA> Eval for StateLocal<Env, S, M, F, MA> {
    type Output = Self;
}

impl<W, S, M, MA> Eval for StateListen<W, S, M, MA> {
    type Output = Self;
}

impl<W, S, M, F, MA> Eval for StateCensor<W, S, M, F, MA> {
    type Output = Self;
}

impl<E, S, M, MA, H> Eval for StateCatch<E, S, M, MA, H> {
    type Output = Self;
}

#[doc(hidden)]
pub trait StateRunner<Init> {
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

impl<S, M, Env> MonadReader<Env> for StateT<S, M>
where
    M: Monad + MonadReader<Env>,
{
    type Ask = StateLift<S, M, <M as MonadReader<Env>>::Ask>;
    type Local<F, MA> = StateLocal<Env, S, M, F, MA>;
}

impl<S, M, W> MonadWriter<W> for StateT<S, M>
where
    M: Monad + MonadWriter<W>,
{
    type Tell<Chunk> = StateLift<S, M, <M as MonadWriter<W>>::Tell<Chunk>>;
    type Listen<MA> = StateListen<W, S, M, MA>;
    type Censor<F, MA> = StateCensor<W, S, M, F, MA>;
}

impl<S, M, E> MonadError<E> for StateT<S, M>
where
    M: Monad + MonadError<E>,
{
    type Throw<Reason> = StateLift<S, M, <M as MonadError<E>>::Throw<Reason>>;
    type Catch<MA, H> = StateCatch<E, S, M, MA, H>;
}

impl<S, M, Req> MonadSuspend<Req> for StateT<S, M>
where
    M: Monad + MonadSuspend<Req>,
{
    type Suspend<Request> = StateLift<S, M, <M as MonadSuspend<Req>>::Suspend<Request>>;
}

impl<Init, S, M, A> StateRunner<Init> for StatePure<S, M, A>
where
    M: Monad,
    Pure<M, Pair<A, Init>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<A, Init>>>;
}

impl<Init, S, M> StateRunner<Init> for StateGet<S, M>
where
    M: Monad,
    Pure<M, Pair<Init, Init>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Init, Init>>>;
}

impl<Init, S, M, NewState> StateRunner<Init> for StatePut<S, M, NewState>
where
    M: Monad,
    NewState: Eval,
    Pure<M, Pair<Unit, Evaluate<NewState>>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, Evaluate<NewState>>>>;
}

impl<Init, S, M, F> StateRunner<Init> for StateModify<S, M, F>
where
    M: Monad,
    F: Op<Init>,
    <F as Op<Init>>::Output: Eval,
    Pure<M, Pair<Unit, Evaluate<<F as Op<Init>>::Output>>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, Evaluate<<F as Op<Init>>::Output>>>>;
}

pub struct LStateLiftCont<M, Init>(pub PhantomData<(M, Init)>);

impl<M, Init, A> Op<A> for LStateLiftCont<M, Init>
where
    M: Monad,
    Pure<M, Pair<A, Init>>: Eval,
{
    type Output = Pure<M, Pair<A, Init>>;
}

impl<Init, S, M, MA> StateRunner<Init> for StateLift<S, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LStateLiftCont<M, Init>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LStateLiftCont<M, Init>>>;
}

pub struct LStateBindCont<K>(pub PhantomData<K>);

impl<A, NextState, K> Op<Pair<A, NextState>> for LStateBindCont<K>
where
    K: Op<A>,
    <K as Op<A>>::Output: Eval,
    Evaluate<<K as Op<A>>::Output>: StateRunner<NextState>,
{
    type Output = <Evaluate<<K as Op<A>>::Output> as StateRunner<NextState>>::Output;
}

impl<Init, S, M, MA, K> StateRunner<Init> for StateBind<S, M, MA, K>
where
    MA: StateRunner<Init>,
    M: Monad,
    Bind<M, <MA as StateRunner<Init>>::Output, LStateBindCont<K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as StateRunner<Init>>::Output, LStateBindCont<K>>>;
}

impl<Init, Env, S, M, F, MA> StateRunner<Init> for StateLocal<Env, S, M, F, MA>
where
    M: Monad + MonadReader<Env>,
    MA: StateRunner<Init>,
    <M as MonadReader<Env>>::Local<F, <MA as StateRunner<Init>>::Output>: Eval,
{
    type Output = Evaluate<<M as MonadReader<Env>>::Local<F, <MA as StateRunner<Init>>::Output>>;
}

pub struct LStateListenCont<M>(pub PhantomData<M>);

impl<M, A, NextState, W> Op<Pair<Pair<A, NextState>, W>> for LStateListenCont<M>
where
    M: Monad,
    Pure<M, Pair<Pair<A, W>, NextState>>: Eval,
{
    type Output = Pure<M, Pair<Pair<A, W>, NextState>>;
}

impl<Init, W, S, M, MA> StateRunner<Init> for StateListen<W, S, M, MA>
where
    M: Monad + MonadWriter<W>,
    MA: StateRunner<Init>,
    <M as MonadWriter<W>>::Listen<<MA as StateRunner<Init>>::Output>: Eval,
    Bind<
        M,
        Evaluate<<M as MonadWriter<W>>::Listen<<MA as StateRunner<Init>>::Output>>,
        LStateListenCont<M>,
    >: Eval,
{
    type Output = Evaluate<
        Bind<
            M,
            Evaluate<<M as MonadWriter<W>>::Listen<<MA as StateRunner<Init>>::Output>>,
            LStateListenCont<M>,
        >,
    >;
}

impl<Init, W, S, M, F, MA> StateRunner<Init> for StateCensor<W, S, M, F, MA>
where
    M: Monad + MonadWriter<W>,
    MA: StateRunner<Init>,
    <M as MonadWriter<W>>::Censor<F, <MA as StateRunner<Init>>::Output>: Eval,
{
    type Output = Evaluate<<M as MonadWriter<W>>::Censor<F, <MA as StateRunner<Init>>::Output>>;
}

pub struct LStateCatchCont<Init, H>(pub PhantomData<(Init, H)>);

impl<Init, H, Reason> Op<Reason> for LStateCatchCont<Init, H>
where
    H: Op<Reason>,
    <H as Op<Reason>>::Output: Eval,
    Evaluate<<H as Op<Reason>>::Output>: StateRunner<Init>,
{
    type Output = <Evaluate<<H as Op<Reason>>::Output> as StateRunner<Init>>::Output;
}

impl<Init, E, S, M, MA, H> StateRunner<Init> for StateCatch<E, S, M, MA, H>
where
    M: Monad + MonadError<E>,
    MA: StateRunner<Init>,
    <M as MonadError<E>>::Catch<<MA as StateRunner<Init>>::Output, LStateCatchCont<Init, H>>: Eval,
{
    type Output = Evaluate<
        <M as MonadError<E>>::Catch<<MA as StateRunner<Init>>::Output, LStateCatchCont<Init, H>>,
    >;
}

#[derive(Debug)]
pub struct RunState<Init, MA>(pub PhantomData<(Init, MA)>);

impl<Init, MA> Eval for RunState<Init, MA>
where
    MA: Eval,
    Evaluate<MA>: StateRunner<Init>,
{
    type Output = <Evaluate<MA> as StateRunner<Init>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{Bind, EitherT, IdK, Ok, RunEither, RunId, Unit};

    struct NewState;
    impl crate::core::Value for NewState {}

    struct SetNewState;
    impl Op<Unit> for SetNewState {
        type Output = NewState;
    }

    #[test]
    fn run_state_get_returns_current_state() {
        type Program = StateGet<Unit, IdK>;
        type Result = Evaluate<RunId<RunState<Unit, Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn run_state_put_updates_state() {
        type Program = StatePut<Unit, IdK, NewState>;
        type Result = Evaluate<RunId<RunState<Unit, Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, NewState>);
    }

    #[test]
    fn run_state_modify_uses_op_to_transform_state() {
        type Program = StateModify<Unit, IdK, SetNewState>;
        type Result = Evaluate<RunId<RunState<Unit, Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, NewState>);
    }

    #[test]
    fn run_state_bind_threads_state() {
        type Program = Bind<
            StateT<Unit, IdK>,
            StateGet<Unit, IdK>,
            super::super::LConst<StatePut<Unit, IdK, NewState>>,
        >;
        type Result = Evaluate<RunId<RunState<Unit, Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, NewState>);
    }

    #[test]
    fn run_state_over_either_stack_evaluates_to_inner_monad() {
        type Program = Pure<StateT<Unit, EitherT<Unit, IdK>>, Unit>;
        type Result = Evaluate<RunId<RunEither<RunState<Unit, Program>>>>;
        assert_type_eq_all!(Result, Ok<Pair<Unit, Unit>>);
    }
}
