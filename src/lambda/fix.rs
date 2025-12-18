use std::marker::PhantomData;

use super::{Apply, Lambda};

// =========================================================================
// Fixed-Point Combinator
// =========================================================================

/// The Fixed-Point Combinator `Fix`
///
/// Rule: Fix<F> x = (F Fix<F>) x
pub struct LFix<F>(PhantomData<F>);
impl<F> Lambda for LFix<F> {
    type Output = LFix<F>;
}

impl<F, X> Apply<X> for LFix<F>
where
    F: Apply<LFix<F>>,
    <F as Apply<LFix<F>>>::Output: Apply<X>,
{
    type Output = <<F as Apply<LFix<F>>>::Output as Apply<X>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{LFalse, LPureIf, LTrue};

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
            Arg: Apply<LTrue>,
            <Arg as Apply<LTrue>>::Output: Apply<Thunk<R, LTrue>>,
        {
            // Use PureIf alias
            type Output = LPureIf<Arg, LTrue, Thunk<R, LTrue>>;
        }

        type F = LFix<LoopBody>;

        type Res1 = App<F, LTrue>;
        assert_type_eq_all!(Res1, LTrue);

        type Res2 = App<F, LFalse>;
        assert_type_eq_all!(Res2, Thunk<F, LTrue>);

        type Res3 = App<Res2, Force>;
        assert_type_eq_all!(Res3, LTrue);
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

        type RecFunc = LFix<Unroll>;

        assert_type_eq_all!(App<RecFunc, Z>, Done);
        assert_type_eq_all!(App<RecFunc, S<Z>>, Done);
        assert_type_eq_all!(App<RecFunc, S<S<Z>>>, Done);
    }
}
