use core::marker::PhantomData;

use crate::core::Value;

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
pub struct Yielded<Request>(pub PhantomData<Request>);

impl Value for Unit {}
impl<A, B> Value for Pair<A, B> {}
impl<A> Value for Ok<A> {}
impl<E> Value for Err<E> {}
impl<A> Value for Done<A> {}
impl<Request> Value for Yielded<Request> {}
