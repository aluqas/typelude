#[test]
fn const_and_add_produce_expected_stack() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpI32Add];
    type Final = Run<InitialState<TTerm, ZeroPages, TTerm, Program>>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<Final as StateProgram>::Output, TTerm);
}

#[test]
fn sub_uses_wasm_operand_order() {
    type Program = tarr![OpI32Const<U5>, OpI32Const<U2>, OpI32Sub];
    type Final = Run<InitialState<TTerm, ZeroPages, TTerm, Program>>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U3>]);
}

#[test]
fn eqz_returns_canonical_i32_boolean() {
    type ZeroProgram = tarr![OpI32Const<U0>, OpI32Eqz];
    type ZeroFinal = Run<InitialState<TTerm, ZeroPages, TTerm, ZeroProgram>>;
    type NonZeroProgram = tarr![OpI32Const<U3>, OpI32Eqz];
    type NonZeroFinal = Run<InitialState<TTerm, ZeroPages, TTerm, NonZeroProgram>>;

    assert_type_eq_all!(<ZeroFinal as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<NonZeroFinal as StateStack>::Output, tarr![WasmI32<U0>]);
}
