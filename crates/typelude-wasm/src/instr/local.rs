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

    #[test]
    fn local_set_and_get_round_trip() {
        type Program = tarr![OpI32Const<U7>, OpLocalSet<U0>, OpLocalGet<U0>];
        type Final = Run<InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>>;

        assert_type_eq_all!(<Final as StateLocals>::Output, tarr![WasmI32<U7>]);
        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U7>]);
    }

    #[test]
    fn local_tee_updates_local_and_preserves_stack() {
        type Program = tarr![OpI32Const<U9>, OpLocalTee<U0>];
        type Final = Run<InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>>;

        assert_type_eq_all!(<Final as StateLocals>::Output, tarr![WasmI32<U9>]);
        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U9>]);
    }
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
