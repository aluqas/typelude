use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{opcode::OpI32Const, run::Step, state::WasmState, value::WasmI32};

impl<Module, Val, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<OpI32Const<Val>, Rest>>,
    >
{
    type Output =
        WasmState<Module, TArr<WasmI32<Val>, Stack>, Locals, Memory, Frames, Branches, Rest>;
}
