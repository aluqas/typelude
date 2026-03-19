use typelude_std::std::prim::option::{None, Some};

use crate::vm::{run::direct::Machine, state::VmState};
pub use crate::{
    shared::trap::{
        BadLocalIndex, BadMemoryIndex, LocalUnderflow, ReturnUnderflow, StackUnderflow,
    },
    vm::step::result::{Continue, Halt, Suspend, Trap},
};

pub mod call;
pub mod control;
pub mod host;
pub mod local;
pub mod memory;
pub mod numeric;
pub mod result;
pub mod stack;

pub trait Step<M> {
    type Output;
}

pub trait ExtractContinue {
    type Output;
}

impl<M> ExtractContinue for Continue<M> {
    type Output = M;
}

pub trait RunNext {
    type Output;
}

impl<M> RunNext for Continue<M>
where
    M: crate::vm::run::direct::RunMachine,
{
    type Output = <M as crate::vm::run::direct::RunMachine>::Output;
}

impl<M> RunNext for Halt<M> {
    type Output = Halt<M>;
}

impl<R, M> RunNext for Trap<R, M> {
    type Output = Trap<R, M>;
}

impl<R, M> RunNext for Suspend<R, M> {
    type Output = Suspend<R, M>;
}

pub trait ToStepResult {
    type Output;
}

impl<M> ToStepResult for Some<M> {
    type Output = Continue<M>;
}

impl ToStepResult for None {
    type Output = None;
}

pub(crate) type CoreMachine<Stack, Locals, Memory, Frames, Labels, Program, Meta, Fx> =
    Machine<VmState<Stack, Locals, Memory, Frames, Labels, Program>, Meta, Fx>;

pub(crate) type StepMachine<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> = Machine<
    VmState<
        Stack,
        Locals,
        Memory,
        Frames,
        Labels,
        typelude_std::std::col::array::Array<Inst, Rest>,
    >,
    Meta,
    Fx,
>;

pub use call::*;
pub use control::*;
pub use local::*;
pub use memory::*;
pub use numeric::*;
pub use stack::*;
