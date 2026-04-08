use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, Op},
    effect::{Monad, Pure},
};

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
    K: Op<<Evaluate<MA> as UnwrapId>::Value>,
    <K as Op<<Evaluate<MA> as UnwrapId>::Value>>::Output: Eval,
{
    type Output = Evaluate<<K as Op<<Evaluate<MA> as UnwrapId>::Value>>::Output>;
}

#[derive(Debug)]
pub struct RunId<MA>(pub PhantomData<MA>);

#[allow(dead_code)]
type _UseAliases = (Pure<IdK, ()>,);

impl<MA> Eval for RunId<MA>
where
    MA: Eval,
    Evaluate<MA>: UnwrapId,
{
    type Output = <Evaluate<MA> as UnwrapId>::Value;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::effect::{Bind, LConst, Pair, Unit};

    struct KViaApply;

    impl Op<Unit> for KViaApply {
        type Output = crate::core::Apply<LConst<Id<Pair<Unit, Unit>>>, Unit>;
    }

    #[test]
    fn run_id_pure_returns_inner_value() {
        type Result = Evaluate<RunId<Pure<IdK, Unit>>>;
        assert_type_eq_all!(Result, Unit);
    }

    #[test]
    fn run_id_bind_evaluates_continuation_expression() {
        type Program = Bind<IdK, Id<Unit>, KViaApply>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }
}
