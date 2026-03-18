use core::marker::PhantomData;

use typelude_std::{core::Eval, std::debug::trace::Trace};

use crate::machine::{core::CoreState, meta::VmMeta};

#[derive(Debug)]
pub struct Machine<Core, Meta, Fx>(pub PhantomData<(Core, Meta, Fx)>);

impl<C, M, F> Eval for Machine<C, M, F> {
    type Output = Self;
}

impl<S, L, M, F, B, P, H, Fuel, W, Fx> Trace
    for Machine<CoreState<S, L, M, F, B, P>, VmMeta<H, Fuel, W>, Fx>
where
    S: Trace,
    L: Trace,
    M: Trace,
    H: Trace,
{
    fn fmt() -> String {
        format!(
            "MachineState {{\n  Stack: {}\n  Locals: {}\n  Memory: {}\n  History: {}\n}}",
            S::fmt(),
            L::fmt(),
            M::fmt(),
            H::fmt()
        )
    }
}
