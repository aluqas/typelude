use core::marker::PhantomData;

use typelude_std::core::{Eval, TyFn};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unit;

#[derive(Debug)]
pub struct Pair<A, B>(pub PhantomData<(A, B)>);

#[derive(Debug)]
pub struct Ok<A>(pub PhantomData<A>);

#[derive(Debug)]
pub struct Err<E>(pub PhantomData<E>);

#[derive(Debug)]
pub struct Done<A>(pub PhantomData<A>);

#[derive(Debug)]
pub struct Yielded<R>(pub PhantomData<R>);

impl Eval for Unit {
    type Output = Self;
}

impl<A, B> Eval for Pair<A, B> {
    type Output = Self;
}

impl<A> Eval for Ok<A> {
    type Output = Self;
}

impl<E> Eval for Err<E> {
    type Output = Self;
}

impl<A> Eval for Done<A> {
    type Output = Self;
}

impl<R> Eval for Yielded<R> {
    type Output = Self;
}

pub trait Monad {
    type Pure<A>;
    type Bind<MA, K>;
}

pub type Pure<F, A> = <F as Monad>::Pure<A>;
pub type Bind<F, MA, K> = <F as Monad>::Bind<MA, K>;

pub trait MonadTrans<Inner>: Monad {
    type Lift<MA>;
}

pub trait MonadState<S>: Monad {
    type Get;
    type Put<NewState>;
    type Modify<F>;
}

pub trait MonadWriter<W>: Monad {
    type Tell<Item>;
}

pub trait MonadError<E>: Monad {
    type Throw<Reason>;
}

pub trait MonadSuspend<R>: Monad {
    type Suspend<Request>;
}

pub struct LConst<T>(pub PhantomData<T>);

impl<A, T> TyFn<A> for LConst<T> {
    type Output = T;
}
