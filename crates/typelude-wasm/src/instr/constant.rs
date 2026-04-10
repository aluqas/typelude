use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{opcode::OpI32Const, run::Step, state::WasmState, value::WasmI32};

impl<Module, Store, Val, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpI32Const<Val>, Rest>>,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmI32<Val>, Stack>, Locals, Frames, Branches, Rest>;
}
