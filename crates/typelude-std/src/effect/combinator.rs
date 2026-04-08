use core::marker::PhantomData;

use crate::{
    core::{Apply, Eval, Evaluate, Op},
    effect::{Bind, LConst, Monad, Pure},
};

pub type Then<F, MA, MB> = Bind<F, MA, LConst<MB>>;

pub type Map<F, MA, K> = Bind<F, MA, LMap<F, K>>;

pub struct LMap<F, K>(pub PhantomData<(F, K)>);

impl<F, K, A> Op<A> for LMap<F, K>
where
    F: Monad,
    Apply<K, A>: Eval,
{
    type Output = Pure<F, Evaluate<Apply<K, A>>>;
}

pub type Join<F, MMA> = Bind<F, MMA, LIdentity>;

pub struct LIdentity;

impl<A> Op<A> for LIdentity {
    type Output = A;
}

pub type Ap<F, MF, MA> = Bind<F, MF, LApLeft<F, MA>>;

pub struct LApLeft<F, MA>(pub PhantomData<(F, MA)>);

impl<F, MA, Func> Op<Func> for LApLeft<F, MA>
where
    F: Monad,
{
    type Output = Bind<F, MA, LApRight<F, Func>>;
}

pub struct LApRight<F, Func>(pub PhantomData<(F, Func)>);

impl<F, Func, A> Op<A> for LApRight<F, Func>
where
    F: Monad,
    Apply<Func, A>: Eval,
{
    type Output = Pure<F, Evaluate<Apply<Func, A>>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::{
        core::Evaluate,
        effect::{Id, IdK, Pair, RunId, Unit},
    };

    struct MakePair;
    impl Op<Unit> for MakePair {
        type Output = Pair<Unit, Unit>;
    }

    struct MakeIdUnit;
    impl Op<Unit> for MakeIdUnit {
        type Output = Id<Unit>;
    }

    struct FnPair;
    impl Op<Unit> for FnPair {
        type Output = Pair<Unit, Unit>;
    }
    impl crate::core::Value for FnPair {}

    #[test]
    fn then_discards_left_value() {
        type Program = Then<IdK, Pure<IdK, Unit>, Pure<IdK, Pair<Unit, Unit>>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn map_transforms_inner_value() {
        type Program = Map<IdK, Pure<IdK, Unit>, MakePair>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn join_flattens_nested_monad() {
        type Program = Join<IdK, Pure<IdK, Id<Unit>>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Unit);
    }

    #[test]
    fn ap_applies_monadic_function_to_monadic_value() {
        type Program = Ap<IdK, Pure<IdK, FnPair>, Pure<IdK, Unit>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn join_works_with_bind_returning_monad() {
        type Program = Join<IdK, Map<IdK, Pure<IdK, Unit>, MakeIdUnit>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Unit);
    }
}
