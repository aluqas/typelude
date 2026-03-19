use core::marker::PhantomData;

use typelude_std::core::TyFn;

use crate::{
    core::traits::{MonadError, MonadState},
    vm::algebra::effect::{
        state_ops::PutVm,
        trap::{ThrowVm, VmTrap},
    },
};

#[derive(Debug)]
pub struct StepOk<State>(pub PhantomData<State>);

#[derive(Debug)]
pub struct StepErr<Reason>(pub PhantomData<Reason>);

pub struct LApplyStep<F, RootState>(pub PhantomData<(F, RootState)>);

impl<F, RootState, NextState> TyFn<StepOk<NextState>> for LApplyStep<F, RootState>
where
    F: MonadState<RootState>,
    PutVm<F, RootState, NextState>: Sized,
{
    type Output = PutVm<F, RootState, NextState>;
}

impl<F, RootState, Reason> TyFn<StepErr<Reason>> for LApplyStep<F, RootState>
where
    F: MonadError<VmTrap>,
{
    type Output = ThrowVm<F, Reason>;
}

pub struct LRunFallible<Func, F, RootState>(pub PhantomData<(Func, F, RootState)>);

impl<Func, F, RootState, CurrentState> TyFn<CurrentState> for LRunFallible<Func, F, RootState>
where
    Func: TyFn<CurrentState>,
    LApplyStep<F, RootState>: TyFn<<Func as TyFn<CurrentState>>::Output>,
{
    type Output =
        <LApplyStep<F, RootState> as TyFn<<Func as TyFn<CurrentState>>::Output>>::Output;
}
