use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{
    opcode::{OpI32Const, OpI64Const},
    run::Step,
    state::WasmState,
    value::{WasmI32, WasmI64},
};

impl<Module, Store, Val, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpI32Const<Val>, Rest>>,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmI32<Val>, Stack>, Locals, Frames, Branches, Rest>;
}

impl<Module, Store, Val, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpI64Const<Val>, Rest>>,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmI64<Val>, Stack>, Locals, Frames, Branches, Rest>;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    include!("../tests/cases/instr_constant.rs");
}
