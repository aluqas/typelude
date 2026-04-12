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
