#[test]
fn invoke_export_resolves_function_exports_by_name() {
    type Main = Fn0<TTerm, tarr![OpI32Const<U1>, OpReturn]>;
    type Module = WasmModule<
        TTerm,
        WasmFuncSpace<tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>], tarr![Main]>,
        WasmModuleMemory<NoMemoryDecl, TTerm>,
        WasmModuleTables<TTerm, TTerm>,
        TTerm,
        tarr![WasmExport<tstr::TS!("main"), ExportFunc<U0>>],
        NoStart
    >;
    type Final = InvokeExport<Module, tstr::TS!("main"), TTerm>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U1>]);
}
