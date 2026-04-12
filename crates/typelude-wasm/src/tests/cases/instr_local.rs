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
