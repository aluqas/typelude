//! **Machine State**
//!
//! Machine state definition (Stack, Memory, CallStack, Program)

use std::marker::PhantomData;

use crate::eval::{Evaluable, Sealed};

/// Machine State
/// - `Stack`: Calculation Stack (TyArray)
/// - `Locals`: Local Variables (TyArray)
/// - `Memory`: Linear Memory (TyArray)
/// - `CallStack`: Call Stack (TyArray of Frames)
/// - `Program`: Currently executing instruction sequence (TyArray of Instructions)
#[derive(Debug)]
pub struct MachineState<Stack, Locals, Memory, CallStack, Program>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program)>,
);

impl<S, L, M, C, P> Sealed for MachineState<S, L, M, C, P> {}

impl<S, L, M, C, P> Evaluable for MachineState<S, L, M, C, P> {
    type Output = MachineState<S, L, M, C, P>;
}
