use std::marker::PhantomData;

use super::{Apply, Lambda};

// =========================================================================
// Fixed-Point Combinator
// =========================================================================

/// The Fixed-Point Combinator `Fix`
///
/// Rule: Fix<F> x = (F Fix<F>) x
pub struct Fix<F>(PhantomData<F>);
impl<F> Lambda for Fix<F> {
    type Output = Fix<F>;
}

impl<F, X> Apply<X> for Fix<F>
where
    F: Apply<Fix<F>>,
    <F as Apply<Fix<F>>>::Output: Apply<X>,
{
    type Output = <<F as Apply<Fix<F>>>::Output as Apply<X>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{False, PureIf, True};

    type App<F, A> = <F as Apply<A>>::Output;

    #[test]
    fn test_loop_basic_unroll() {
        struct Thunk<F, A>(std::marker::PhantomData<(F, A)>);

        struct Force;
        impl<F, A> Apply<Force> for Thunk<F, A>
        where
            F: Apply<A>,
        {
            type Output = <F as Apply<A>>::Output;
        }

        struct LoopBody;
        struct LoopBody1<R>(std::marker::PhantomData<R>);

        impl<R> Apply<R> for LoopBody {
            type Output = LoopBody1<R>;
        }

        impl<R, Arg> Apply<Arg> for LoopBody1<R>
        where
            Arg: Apply<True>,
            <Arg as Apply<True>>::Output: Apply<Thunk<R, True>>,
        {
            // Use PureIf alias
            type Output = PureIf<Arg, True, Thunk<R, True>>;
        }

        type F = Fix<LoopBody>;

        type Res1 = App<F, True>;
        assert_type_eq_all!(Res1, True);

        type Res2 = App<F, False>;
        assert_type_eq_all!(Res2, Thunk<F, True>);

        type Res3 = App<Res2, Force>;
        assert_type_eq_all!(Res3, True);
    }

    #[test]
    fn test_church_factorial_ish() {
        struct Z;
        struct S<N>(std::marker::PhantomData<N>);

        struct Unroll;
        struct Unroll1<R>(std::marker::PhantomData<R>);

        impl<R> Apply<R> for Unroll {
            type Output = Unroll1<R>;
        }

        struct Done;

        impl<R> Apply<Z> for Unroll1<R> {
            type Output = Done;
        }

        impl<R, P> Apply<S<P>> for Unroll1<R>
        where
            R: Apply<P>,
        {
            type Output = <R as Apply<P>>::Output;
        }

        type RecFunc = Fix<Unroll>;

        assert_type_eq_all!(App<RecFunc, Z>, Done);
        assert_type_eq_all!(App<RecFunc, S<Z>>, Done);
        assert_type_eq_all!(App<RecFunc, S<S<Z>>>, Done);
    }
}
