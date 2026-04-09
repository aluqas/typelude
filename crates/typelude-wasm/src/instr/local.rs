use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{
    helpers::local_index::{LocalGet, LocalSet},
    opcode::{OpLocalGet, OpLocalSet, OpLocalTee},
    run::Step,
    state::WasmState,
};

impl<Idx, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<OpLocalGet<Idx>, Rest>>>
where
    Locals: LocalGet<Idx>,
{
    type Output = WasmState<
        TArr<<Locals as LocalGet<Idx>>::Output, Stack>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Idx, Value, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            TArr<Value, Stack>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpLocalSet<Idx>, Rest>,
        >,
    >
where
    Locals: LocalSet<Idx, Value>,
{
    type Output =
        WasmState<Stack, <Locals as LocalSet<Idx, Value>>::Output, Memory, Frames, Branches, Rest>;
}

impl<Idx, Value, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            TArr<Value, Stack>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpLocalTee<Idx>, Rest>,
        >,
    >
where
    Locals: LocalSet<Idx, Value>,
{
    type Output = WasmState<
        TArr<Value, Stack>,
        <Locals as LocalSet<Idx, Value>>::Output,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}
