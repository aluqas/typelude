use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, TyFn},
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::composed::core::traits::{Bind, Monad, MonadTrans, MonadWriter, Pair, Pure, Unit};

#[derive(Debug)]
pub struct WriterT<W, M>(pub PhantomData<(W, M)>);

#[derive(Debug)]
pub struct WriterPure<W, M, A>(pub PhantomData<(W, M, A)>);

#[derive(Debug)]
pub struct WriterBind<W, M, MA, K>(pub PhantomData<(W, M, MA, K)>);

#[derive(Debug)]
pub struct WriterTell<W, M, Item>(pub PhantomData<(W, M, Item)>);

#[derive(Debug)]
pub struct WriterLift<W, M, MA>(pub PhantomData<(W, M, MA)>);

impl<W, M, A> Eval for WriterPure<W, M, A> {
    type Output = Self;
}

impl<W, M, MA, K> Eval for WriterBind<W, M, MA, K> {
    type Output = Self;
}

impl<W, M, Item> Eval for WriterTell<W, M, Item> {
    type Output = Self;
}

impl<W, M, MA> Eval for WriterLift<W, M, MA> {
    type Output = Self;
}

pub trait RunWriter {
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
    type Tell<Item> = WriterTell<W, M, Item>;
}

impl<W, M, A> RunWriter for WriterPure<W, M, A>
where
    M: Monad,
    Pure<M, Pair<A, Nil>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<A, Nil>>>;
}

impl<W, M, Item> RunWriter for WriterTell<W, M, Item>
where
    M: Monad,
    Pure<M, Pair<Unit, Array<Item, Nil>>>: Eval,
{
    type Output = Evaluate<Pure<M, Pair<Unit, Array<Item, Nil>>>>;
}

pub struct LWriterLiftCont<M>(pub PhantomData<M>);

impl<M, A> TyFn<A> for LWriterLiftCont<M>
where
    M: Monad,
    Pure<M, Pair<A, Nil>>: Eval,
{
    type Output = Pure<M, Pair<A, Nil>>;
}

impl<W, M, MA> RunWriter for WriterLift<W, M, MA>
where
    M: Monad,
    MA: Eval,
    Bind<M, Evaluate<MA>, LWriterLiftCont<M>>: Eval,
{
    type Output = Evaluate<Bind<M, Evaluate<MA>, LWriterLiftCont<M>>>;
}

pub struct LWriterAppendCont<M, Log1>(pub PhantomData<(M, Log1)>);

impl<M, Log1, A, Log2> TyFn<Pair<A, Log2>> for LWriterAppendCont<M, Log1>
where
    M: Monad,
    Log1: Concat<Log2> + IsList,
    Log2: IsList,
    Pure<M, Pair<A, <Log1 as Concat<Log2>>::Output>>: Eval,
{
    type Output = Pure<M, Pair<A, <Log1 as Concat<Log2>>::Output>>;
}

pub struct LWriterBindCont<M, K>(pub PhantomData<(M, K)>);

impl<M, K, A, Log1> TyFn<Pair<A, Log1>> for LWriterBindCont<M, K>
where
    M: Monad,
    Log1: IsList,
    K: TyFn<A>,
    <K as TyFn<A>>::Output: RunWriter,
    Bind<M, <<K as TyFn<A>>::Output as RunWriter>::Output, LWriterAppendCont<M, Log1>>: Eval,
{
    type Output = Bind<M, <<K as TyFn<A>>::Output as RunWriter>::Output, LWriterAppendCont<M, Log1>>;
}

impl<W, M, MA, K> RunWriter for WriterBind<W, M, MA, K>
where
    M: Monad,
    MA: RunWriter,
    Bind<M, <MA as RunWriter>::Output, LWriterBindCont<M, K>>: Eval,
{
    type Output = Evaluate<Bind<M, <MA as RunWriter>::Output, LWriterBindCont<M, K>>>;
}

pub struct ERunWriter<MA>(pub PhantomData<MA>);

impl<MA> Eval for ERunWriter<MA>
where
    MA: Eval,
    Evaluate<MA>: RunWriter,
{
    type Output = <Evaluate<MA> as RunWriter>::Output;
}
