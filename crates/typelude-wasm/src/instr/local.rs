use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{
    helpers::local_index::{LocalGet, LocalSet},
    opcode::{OpLocalGet, OpLocalSet, OpLocalTee},
    run::Step,
    state::WasmState,
};

impl<Module, Store, Idx, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpLocalGet<Idx>, Rest>>,
    >
where
    Locals: LocalGet<Idx>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<Locals as LocalGet<Idx>>::Output, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Idx, ValueT, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<ValueT, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpLocalSet<Idx>, Rest>,
        >,
    >
where
    Locals: LocalSet<Idx, ValueT>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        <Locals as LocalSet<Idx, ValueT>>::Output,
        Frames,
        Branches,
        Rest,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    include!("../tests/cases/instr_local.rs");
}

impl<Module, Store, Idx, ValueT, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<ValueT, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpLocalTee<Idx>, Rest>,
        >,
    >
where
    Locals: LocalSet<Idx, ValueT>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<ValueT, Stack>,
        <Locals as LocalSet<Idx, ValueT>>::Output,
        Frames,
        Branches,
        Rest,
    >;
}
