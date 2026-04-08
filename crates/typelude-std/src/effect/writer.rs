use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, Op};

use super::{
    Append, Bind, Empty, Monad, MonadError, MonadReader, MonadState, MonadSuspend, MonadTrans,
    MonadWriter, Pair, Pure, Unit,
};

#[derive(Debug)]
pub struct WriterT<W, M>(pub PhantomData<(W, M)>);

#[derive(Debug)]
pub struct WriterPure<W, M, A>(pub PhantomData<(W, M, A)>);

#[derive(Debug)]
pub struct WriterBind<W, M, MA, K>(pub PhantomData<(W, M, MA, K)>);

#[derive(Debug)]
pub struct WriterTell<W, M, Chunk>(pub PhantomData<(W, M, Chunk)>);

#[derive(Debug)]
pub struct WriterLift<W, M, MA>(pub PhantomData<(W, M, MA)>);

#[derive(Debug)]
pub struct WriterListen<W, M, MA>(pub PhantomData<(W, M, MA)>);

#[derive(Debug)]
pub struct WriterCensor<W, M, F, MA>(pub PhantomData<(W, M, F, MA)>);

#[derive(Debug)]
pub struct WriterLocal<Env, W, M, F, MA>(pub PhantomData<(Env, W, M, F, MA)>);

#[derive(Debug)]
pub struct WriterCatch<E, W, M, MA, H>(pub PhantomData<(E, W, M, MA, H)>);

impl<W, M, A> Eval for WriterPure<W, M, A> {
    type Output = Self;
}

impl<W, M, MA, K> Eval for WriterBind<W, M, MA, K> {
    type Output = Self;
}

impl<W, M, Chunk> Eval for WriterTell<W, M, Chunk> {
    type Output = Self;
}

impl<W, M, MA> Eval for WriterLift<W, M, MA> {
    type Output = Self;
}

impl<W, M, MA> Eval for WriterListen<W, M, MA> {
    type Output = Self;
}

impl<W, M, F, MA> Eval for WriterCensor<W, M, F, MA> {
    type Output = Self;
}

impl<Env, W, M, F, MA> Eval for WriterLocal<Env, W, M, F, MA> {
    type Output = Self;
}

impl<E, W, M, MA, H> Eval for WriterCatch<E, W, M, MA, H> {
    type Output = Self;
}

#[doc(hidden)]
pub trait WriterRunner {
    type Output;
}

impl<W, M> Monad for WriterT<W, M> {
    type Pure<A> = WriterPure<W, M, A>;
    type Bind<MA, K> = WriterBind<W, M, MA, K>;
}

impl<W, M> MonadTrans<M> for WriterT<W, M>
where
    M: Monad,
{
    type Lift<MA> = WriterLift<W, M, MA>;
}

impl<W, M> MonadWriter<W> for WriterT<W, M>
where
    M: Monad,
{
    type Tell<Chunk> = WriterTell<W, M, Chunk>;
    type Listen<MA> = WriterListen<W, M, MA>;
    type Censor<F, MA> = WriterCensor<W, M, F, MA>;
}

impl<W, M, S> MonadState<S> for WriterT<W, M>
where
    M: Monad + MonadState<S>,
{
    type Get = WriterLift<W, M, <M as MonadState<S>>::Get>;
    type Put<NewState> = WriterLift<W, M, <M as MonadState<S>>::Put<NewState>>;
    type Modify<F> = WriterLift<W, M, <M as MonadState<S>>::Modify<F>>;
}

impl<W, M, Env> MonadReader<Env> for WriterT<W, M>
where
    M: Monad + MonadReader<Env>,
{
    type Ask = WriterLift<W, M, <M as MonadReader<Env>>::Ask>;
    type Local<F, MA> = WriterLocal<Env, W, M, F, MA>;
}

impl<W, M, E> MonadError<E> for WriterT<W, M>
where
    M: Monad + MonadError<E>,
{
    type Throw<Reason> = WriterLift<W, M, <M as MonadError<E>>::Throw<Reason>>;
    type Catch<MA, H> = WriterCatch<E, W, M, MA, H>;
}

impl<W, M, Req> MonadSuspend<Req> for WriterT<W, M>
where
    M: Monad + MonadSuspend<Req>,
{
    type Suspend<Request> = WriterLift<W, M, <M as MonadSuspend<Req>>::Suspend<Request>>;
}

impl<W, M, A> WriterRunner for WriterPure<W, M, A>
where
    W: Empty,
    M: Monad,
    Pure<M, Pair<A, <W as Empty>::Output>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<A, <W as Empty>::Output>>>;
}

impl<W, M, Chunk> WriterRunner for WriterTell<W, M, Chunk>
where
    M: Monad,
    Chunk: Eval,
    Pure<M, Pair<Unit, Evaluate<Chunk>>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, Evaluate<Chunk>>>>;
}

pub struct LWriterLiftCont<M, W>(pub PhantomData<(M, W)>);

impl<M, W, A> Op<A> for LWriterLiftCont<M, W>
where
    W: Empty,
    M: Monad,
    Pure<M, Pair<A, <W as Empty>::Output>>: Eval,
{
    type Output = Pure<M, Pair<A, <W as Empty>::Output>>;
}

impl<W, M, MA> WriterRunner for WriterLift<W, M, MA>
where
    W: Empty,
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LWriterLiftCont<M, W>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LWriterLiftCont<M, W>>>;
}

pub struct LWriterAppendCont<M, Log1>(pub PhantomData<(M, Log1)>);

impl<M, Log1, A, Log2> Op<Pair<A, Log2>> for LWriterAppendCont<M, Log1>
where
    M: Monad,
    Log1: Append<Log2>,
    Pure<M, Pair<A, <Log1 as Append<Log2>>::Output>>: Eval,
{
    type Output = Pure<M, Pair<A, <Log1 as Append<Log2>>::Output>>;
}

pub struct LWriterBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A, Log1> Op<Pair<A, Log1>> for LWriterBindCont<M, K>
where
    M: Monad,
    K: Op<A>,
    <K as Op<A>>::Output: Eval,
    Evaluate<<K as Op<A>>::Output>: WriterRunner,
    Bind<M, <Evaluate<<K as Op<A>>::Output> as WriterRunner>::Output, LWriterAppendCont<M, Log1>>:
        Eval,
{
    type Output = Bind<
        M,
        <Evaluate<<K as Op<A>>::Output> as WriterRunner>::Output,
        LWriterAppendCont<M, Log1>,
    >;
}

impl<W, M, MA, K> WriterRunner for WriterBind<W, M, MA, K>
where
    M: Monad,
    MA: WriterRunner,
    Bind<M, <MA as WriterRunner>::Output, LWriterBindCont<M, K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as WriterRunner>::Output, LWriterBindCont<M, K>>>;
}

pub struct LWriterListenCont<M>(pub PhantomData<M>);

impl<M, A, Log> Op<Pair<A, Log>> for LWriterListenCont<M>
where
    M: Monad,
    Pure<M, Pair<Pair<A, Log>, Log>>: Eval,
{
    type Output = Pure<M, Pair<Pair<A, Log>, Log>>;
}

impl<W, M, MA> WriterRunner for WriterListen<W, M, MA>
where
    M: Monad,
    MA: WriterRunner,
    Bind<M, <MA as WriterRunner>::Output, LWriterListenCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as WriterRunner>::Output, LWriterListenCont<M>>>;
}

pub struct LWriterCensorCont<M, F>(pub PhantomData<(M, F)>);

impl<M, F, A, Log> Op<Pair<A, Log>> for LWriterCensorCont<M, F>
where
    M: Monad,
    F: Op<Log>,
    <F as Op<Log>>::Output: Eval,
    Pure<M, Pair<A, Evaluate<<F as Op<Log>>::Output>>>: Eval,
{
    type Output = Pure<M, Pair<A, Evaluate<<F as Op<Log>>::Output>>>;
}

impl<W, M, F, MA> WriterRunner for WriterCensor<W, M, F, MA>
where
    M: Monad,
    MA: WriterRunner,
    Bind<M, <MA as WriterRunner>::Output, LWriterCensorCont<M, F>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as WriterRunner>::Output, LWriterCensorCont<M, F>>>;
}

impl<Env, W, M, F, MA> WriterRunner for WriterLocal<Env, W, M, F, MA>
where
    M: Monad + MonadReader<Env>,
    MA: WriterRunner,
    <M as MonadReader<Env>>::Local<F, <MA as WriterRunner>::Output>: Eval,
{
    type Output = Evaluate<<M as MonadReader<Env>>::Local<F, <MA as WriterRunner>::Output>>;
}

pub struct LWriterCatchCont<H>(pub PhantomData<H>);

impl<H, Reason> Op<Reason> for LWriterCatchCont<H>
where
    H: Op<Reason>,
    <H as Op<Reason>>::Output: Eval,
    Evaluate<<H as Op<Reason>>::Output>: WriterRunner,
{
    type Output = <Evaluate<<H as Op<Reason>>::Output> as WriterRunner>::Output;
}

impl<E, W, M, MA, H> WriterRunner for WriterCatch<E, W, M, MA, H>
where
    M: Monad + MonadError<E>,
    MA: WriterRunner,
    <M as MonadError<E>>::Catch<<MA as WriterRunner>::Output, LWriterCatchCont<H>>: Eval,
{
    type Output =
        Evaluate<<M as MonadError<E>>::Catch<<MA as WriterRunner>::Output, LWriterCatchCont<H>>>;
}

#[derive(Debug)]
pub struct RunWriter<MA>(pub PhantomData<MA>);

impl<MA> Eval for RunWriter<MA>
where
    MA: Eval,
    Evaluate<MA>: WriterRunner,
{
    type Output = <Evaluate<MA> as WriterRunner>::Output;
}

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{Bind, EitherT, IdK, Ok, RunEither, RunId, RunState, StateT, Unit};

    struct TestLogs;
    struct LogNil;
    struct LogCons<Head, Tail>(PhantomData<(Head, Tail)>);

    impl crate::core::Value for LogNil {}
    impl<Head, Tail> crate::core::Value for LogCons<Head, Tail> {}

    impl Empty for TestLogs {
        type Output = LogNil;
    }

    impl<Rhs> Append<Rhs> for LogNil {
        type Output = Rhs;
    }

    impl<Head, Tail, Rhs> Append<Rhs> for LogCons<Head, Tail>
    where
        Tail: Append<Rhs>,
    {
        type Output = LogCons<Head, <Tail as Append<Rhs>>::Output>;
    }

    struct ChunkA;
    struct ChunkB;
    impl crate::core::Value for ChunkA {}
    impl crate::core::Value for ChunkB {}

    struct DropLogs;
    impl Op<LogCons<ChunkA, LogNil>> for DropLogs {
        type Output = LogNil;
    }

    #[test]
    fn run_writer_pure_uses_empty_log() {
        type Program = Pure<WriterT<TestLogs, IdK>, Unit>;
        type Result = Evaluate<RunId<RunWriter<Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, LogNil>);
    }

    #[test]
    fn run_writer_tell_emits_chunk_as_is() {
        type Program = WriterTell<TestLogs, IdK, LogCons<ChunkA, LogNil>>;
        type Result = Evaluate<RunId<RunWriter<Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, LogCons<ChunkA, LogNil>>);
    }

    #[test]
    fn run_writer_bind_appends_logs() {
        type Program = Bind<
            WriterT<TestLogs, IdK>,
            WriterTell<TestLogs, IdK, LogCons<ChunkA, LogNil>>,
            super::super::LConst<WriterTell<TestLogs, IdK, LogCons<ChunkB, LogNil>>>,
        >;
        type Result = Evaluate<RunId<RunWriter<Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, LogCons<ChunkA, LogCons<ChunkB, LogNil>>>);
    }

    #[test]
    fn run_writer_listen_returns_value_and_log() {
        type Program =
            WriterListen<TestLogs, IdK, WriterTell<TestLogs, IdK, LogCons<ChunkA, LogNil>>>;
        type Result = Evaluate<RunId<RunWriter<Program>>>;
        assert_type_eq_all!(
            Result,
            Pair<Pair<Unit, LogCons<ChunkA, LogNil>>, LogCons<ChunkA, LogNil>>
        );
    }

    #[test]
    fn run_writer_censor_rewrites_log() {
        type Program = WriterCensor<
            TestLogs,
            IdK,
            DropLogs,
            WriterTell<TestLogs, IdK, LogCons<ChunkA, LogNil>>,
        >;
        type Result = Evaluate<RunId<RunWriter<Program>>>;
        assert_type_eq_all!(Result, Pair<Unit, LogNil>);
    }

    #[test]
    fn run_writer_over_state_either_id_stack() {
        type Program = Pure<WriterT<TestLogs, StateT<Unit, EitherT<Unit, IdK>>>, Unit>;
        type Result = Evaluate<RunId<RunEither<RunState<Unit, RunWriter<Program>>>>>;
        assert_type_eq_all!(Result, Ok<Pair<Pair<Unit, LogNil>, Unit>>);
    }
}
