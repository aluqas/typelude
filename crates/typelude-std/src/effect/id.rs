use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, TyFn};

use super::{Monad, Pure};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdK;

#[derive(Debug)]
pub struct Id<A>(pub PhantomData<A>);

#[derive(Debug)]
pub struct IdBind<MA, K>(pub PhantomData<(MA, K)>);

pub trait UnwrapId {
    type Value;
}

impl<A> UnwrapId for Id<A> {
    type Value = A;
}

impl Monad for IdK {
    type Pure<A> = Id<A>;
    type Bind<MA, K> = IdBind<MA, K>;
}

impl<A> Eval for Id<A>
where
    A: Eval,
{
    type Output = Id<Evaluate<A>>;
}

impl<MA, K> Eval for IdBind<MA, K>
where
    MA: Eval,
    Evaluate<MA>: UnwrapId,
    K: TyFn<<Evaluate<MA> as UnwrapId>::Value>,
    <K as TyFn<<Evaluate<MA> as UnwrapId>::Value>>::Output: Eval,
{
    type Output = Evaluate<<K as TyFn<<Evaluate<MA> as UnwrapId>::Value>>::Output>;
}

#[allow(dead_code)]
type _UseAliases = (Pure<IdK, ()>,);
