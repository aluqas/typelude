use std::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate};

use super::LApp;
/// The Fixed-Point Combinator `Fix`
///
/// Rule: Fix<F> x = (F Fix<F>) x
pub struct LFix<F>(PhantomData<F>);
impl<F> Eval for LFix<F> {
    type Output = LFix<F>;
}

// Fix<F> X -> F Fix<F> X
impl<F, X> Eval for LApp<LFix<F>, X>
where
    F: Eval + Clone,
    X: Eval,
    // (F Fix<F>)
    LApp<F, LFix<F>>: Eval,
    // (F Fix<F>) X
    LApp<Evaluate<LApp<F, LFix<F>>>, X>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<F, LFix<F>>>, X>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::Evaluate; // Imported here for tests

    use super::*;
    use crate::lambda::church::{LFalse, LTrue};

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    #[test]
    fn test_loop_basic_unroll() {
        struct Thunk<F, A>(std::marker::PhantomData<(F, A)>);
        impl<F, A> Eval for Thunk<F, A> {
            type Output = Thunk<F, A>;
        }

        struct Force;
        impl Eval for Force {
            type Output = Force;
        }

        // Thunk<F, A> Force -> F A
        impl<F, A> Eval for LApp<Thunk<F, A>, Force>
        where
            F: Eval,
            A: Eval,
            LApp<F, A>: Eval,
        {
            type Output = Evaluate<LApp<F, A>>;
        }

        #[derive(Clone)]
        struct LoopBody;
        impl Eval for LoopBody {
            type Output = LoopBody;
        }

        struct LoopBody1<R>(std::marker::PhantomData<R>);
        impl<R> Eval for LoopBody1<R> {
            type Output = LoopBody1<R>;
        }

        // LoopBody R -> LoopBody1<R>
        impl<R> Eval for LApp<LoopBody, R>
        where
            R: Eval,
        {
            type Output = LoopBody1<Evaluate<R>>;
        }

        // LoopBody1<R> Arg -> If Arg True (Thunk R True)
        impl<R, Arg> Eval for LApp<LoopBody1<R>, Arg>
        where
            R: Eval,
            Arg: Eval,
            // LPureIf<Arg, LTrue, Thunk<R, LTrue>>
            // Arg LTrue (Thunk<R, LTrue>)
            LApp<Arg, LTrue>: Eval,
            LApp<Evaluate<LApp<Arg, LTrue>>, Thunk<R, LTrue>>: Eval,
        {
            type Output =
                Evaluate<LApp<Evaluate<LApp<Arg, LTrue>>, Thunk<R, LTrue>>>;
        }

        type F = LFix<LoopBody>;

        // F True -> LoopBody F True -> LoopBody1<F> True -> True True (Thunk F True) ->
        // True
        type Res1 = App<F, LTrue>;
        assert_type_eq_all!(Res1, LTrue);

        // F False -> LoopBody F False -> LoopBody1<F> False -> False True (Thunk F
        // True) -> Thunk F True
        type Res2 = App<F, LFalse>;
        assert_type_eq_all!(Res2, Thunk<F, LTrue>);

        // Force Res2 -> F True -> True
        type Res3 = App<Res2, Force>;
        assert_type_eq_all!(Res3, LTrue);
    }

    #[test]
    fn test_church_factorial_ish() {
        #[derive(Clone)]
        struct Z;
        impl Eval for Z {
            type Output = Z;
        }

        #[derive(Clone)]
        struct S<N>(std::marker::PhantomData<N>);
        impl<N> Eval for S<N> {
            type Output = S<N>;
        }

        #[derive(Clone)]
        struct Unroll;
        impl Eval for Unroll {
            type Output = Unroll;
        }

        struct Unroll1<R>(std::marker::PhantomData<R>);
        impl<R> Eval for Unroll1<R> {
            type Output = Unroll1<R>;
        }

        // Unroll R -> Unroll1<R>
        impl<R> Eval for LApp<Unroll, R>
        where
            R: Eval,
        {
            type Output = Unroll1<Evaluate<R>>;
        }

        #[derive(Clone)]
        struct Done;
        impl Eval for Done {
            type Output = Done;
        }

        // Unroll1<R> Z -> Done
        impl<R> Eval for LApp<Unroll1<R>, Z>
        where
            R: Eval,
        {
            type Output = Done;
        }

        // Unroll1<R> S<P> -> R P
        impl<R, P> Eval for LApp<Unroll1<R>, S<P>>
        where
            R: Eval,
            P: Eval,
            LApp<R, P>: Eval,
        {
            type Output = Evaluate<LApp<R, P>>;
        }

        type RecFunc = LFix<Unroll>;

        // RecFunc Z -> Unroll RecFunc Z -> Unroll1<RecFunc> Z -> Done
        assert_type_eq_all!(App<RecFunc, Z>, Done);

        // RecFunc S<Z> -> Unroll RecFunc S<Z> -> Unroll1<RecFunc> S<Z> -> RecFunc Z ->
        // Done
        assert_type_eq_all!(App<RecFunc, S<Z>>, Done);

        // RecFunc S<S<Z>> -> ... -> Done
        assert_type_eq_all!(App<RecFunc, S<S<Z>>>, Done);
    }
}
