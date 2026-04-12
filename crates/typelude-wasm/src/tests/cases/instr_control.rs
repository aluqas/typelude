type AddTwoAndReturn = Fn0<TTerm, tarr![OpI32Const<U2>, OpI32Add, OpReturn]>;
type SimpleCallModule = Module<tarr![AddTwoAndReturn], U0>;

type ReturnsOne = WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U1>, OpReturn]>;
type ReturnsTwo = WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U2>, OpReturn]>;
type IndirectTypes = tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>];
type DefaultTableDecls = tarr![WasmTableDecl<U2, U2>];
type DefaultElemSegments =
    tarr![WasmElemSegment<U0, WasmConstExpr<tarr![OpI32Const<U0>]>, tarr![U0, U1]>];
type DefaultTableModule =
    ModuleWithTables<tarr![ReturnsOne, ReturnsTwo], IndirectTypes, DefaultTableDecls, DefaultElemSegments>;

#[test]
fn simple_call_and_return_resume_caller_continuation() {
    type Program = tarr![OpI32Const<U3>, OpCall<U0>, OpI32Add];
    type Final = ModuleProgramRun<SimpleCallModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
}

#[test]
fn if_false_branch_executes_else_program() {
    type Program = tarr![OpI32Const<U0>, OpIf<tarr![OpI32Const<U1>], tarr![OpI32Const<U2>]>];
    type Final = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn loop_and_br_if_drive_countdown_to_zero() {
    type Program = tarr![
        OpI32Const<U3>,
        OpLoop<
            tarr![
                OpLocalTee<U0>,
                OpI32Eqz,
                OpBrIf<U1>,
                OpLocalGet<U0>,
                OpI32Const<U1>,
                OpI32Sub,
                OpBr<U0>
            ]
        >
    ];
    type Final = Run<InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>>;

    assert_type_eq_all!(<Final as StateLocals>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn call_indirect_dispatches_through_default_table() {
    type Program = tarr![OpI32Const<U1>, OpCallIndirect<U0>];
    type Final = ModuleProgramRun<DefaultTableModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>]);
}
