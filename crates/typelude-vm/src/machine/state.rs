//! **Machine State**
//!
//! Machine state definition (Stack, Memory, CallStack, Program)

use std::marker::PhantomData;

use typelude_core::Eval;

/// Machine State
/// - `Stack`: Calculation Stack (Array)
/// - `Locals`: Local Variables (Array)
/// - `Memory`: Linear Memory (Array)
/// - `CallStack`: Call Stack (Array of Frames)
/// - `Program`: Currently executing instruction sequence (Array of Instructions)
#[derive(Debug)]
pub struct MachineState<Stack, Locals, Memory, CallStack, Program>(
    pub PhantomData<(Stack, Locals, Memory, CallStack, Program)>,
);

impl<S, L, M, C, P> Eval for MachineState<S, L, M, C, P> {
    type Output = MachineState<S, L, M, C, P>;
}
