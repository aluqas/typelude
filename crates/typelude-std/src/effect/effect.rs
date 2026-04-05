use core::marker::PhantomData;

use typelude_std::core::Eval;

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
