use typelude_col::TArr;
use typelude_std::core::{Eval, Get, Set};

use crate::{
    module::GlobalMut,
    opcode::{OpGlobalGet, OpGlobalSet},
    run::Step,
    state::{WasmGlobal, WasmState, WasmStore},
};

/// global 実体から stack に積む値型を取り出す helper。
pub trait GlobalGetValue {
    type Output;
}

impl<Mutability, ValueT> GlobalGetValue for WasmGlobal<Mutability, ValueT> {
    type Output = ValueT;
}

/// mutable global に新しい値を書き込んだ後の型を作る helper。
pub trait MutableGlobalWith<ValueT> {
    type Output;
}

impl<ValueT, OldValue> MutableGlobalWith<ValueT> for WasmGlobal<GlobalMut, OldValue> {
    type Output = WasmGlobal<GlobalMut, ValueT>;
}

impl<Module, Memory, Tables, Globals, Idx, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            Stack,
            Locals,
            Frames,
            Branches,
            TArr<OpGlobalGet<Idx>, Rest>,
        >,
    >
where
    Globals: Get<Idx>,
    <Globals as Get<Idx>>::Output: GlobalGetValue,
{
    type Output = WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        TArr<<<Globals as Get<Idx>>::Output as GlobalGetValue>::Output, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Memory, Tables, Globals, Idx, ValueT, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<ValueT, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpGlobalSet<Idx>, Rest>,
        >,
    >
where
    Globals: Get<Idx>,
    <Globals as Get<Idx>>::Output: MutableGlobalWith<ValueT>,
    Globals: Set<Idx, <<Globals as Get<Idx>>::Output as MutableGlobalWith<ValueT>>::Output>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            Memory,
            Tables,
            <Globals as Set<
                Idx,
                <<Globals as Get<Idx>>::Output as MutableGlobalWith<ValueT>>::Output,
            >>::Output,
        >,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    type ConstGlobalDecls = tarr![WasmGlobalDecl<GlobalConst, InitI32Const<U5>>];
    type ConstGlobalModule = ModuleWithGlobals<TTerm, U0, ConstGlobalDecls>;
    type MutableGlobalDecls = tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U3>>];
    type MutableGlobalModule = ModuleWithGlobals<TTerm, U0, MutableGlobalDecls>;

    #[test]
    fn const_global_initialized_with_i32_const_is_observable_via_global_get() {
        type Program = tarr![OpGlobalGet<U0>];
        type Final = ModuleProgramRun<ConstGlobalModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
    }

    #[test]
    fn mutable_global_initialized_with_i32_const_is_observable_via_global_get() {
        type Program = tarr![OpGlobalGet<U0>];
        type Final = ModuleProgramRun<MutableGlobalModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U3>]);
    }

    #[test]
    fn global_set_updates_mutable_global() {
        type Program = tarr![OpI32Const<U9>, OpGlobalSet<U0>, OpGlobalGet<U0>];
        type Final = ModuleProgramRun<MutableGlobalModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U9>]);
    }
}
