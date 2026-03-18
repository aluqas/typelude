use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};

use crate::composed::core::{Bind, Monad, Pure, Unit};

#[derive(Debug)]
pub struct EInterpProgram<Prog, F>(pub PhantomData<(Prog, F)>);

pub trait InterpProgram<F> {
    type Output;
}

impl<F> InterpProgram<F> for Nil
where
    F: Monad,
    Pure<F, Unit>: Eval,
{
    type Output = Evaluate<Pure<F, Unit>>;
}

pub struct LProgramThen<Rest, F>(pub PhantomData<(Rest, F)>);

impl<Rest, F, A> typelude_std::core::TyFn<A> for LProgramThen<Rest, F>
where
    Rest: InterpProgram<F>,
{
    type Output = <Rest as InterpProgram<F>>::Output;
}

impl<Inst, Rest, F> InterpProgram<F> for Array<Inst, Rest>
where
    Rest: IsList + InterpProgram<F>,
    Inst: crate::composed::interpret::InterpInstr<F>,
    F: Monad,
    Bind<F, <Inst as crate::composed::interpret::InterpInstr<F>>::Output, LProgramThen<Rest, F>>:
        Eval,
{
    type Output = Evaluate<
        Bind<
            F,
            <Inst as crate::composed::interpret::InterpInstr<F>>::Output,
            LProgramThen<Rest, F>,
        >,
    >;
}

impl<Prog, F> Eval for EInterpProgram<Prog, F>
where
    Prog: InterpProgram<F>,
{
    type Output = <Prog as InterpProgram<F>>::Output;
}
