type ImportAddMain = Fn2<TTerm, tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpCall<U0>, OpReturn]>;
type ImportAddModule = WasmModule<
    tarr![WasmImport<tstr::TS!("host"), tstr::TS!("add"), ImportFunc<WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>>>],
    WasmFuncSpace<tarr![WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>], tarr![ImportAddMain]>,
    WasmModuleMemory<NoMemoryDecl, TTerm>,
    WasmModuleTables<TTerm, TTerm>,
    TTerm,
    tarr![WasmExport<tstr::TS!("main"), ExportFunc<U1>>],
    NoStart
>;

struct HostAdd;

impl HostCall<WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>, Store<ZeroPages>, tarr![WasmI32<U2>, tarr![WasmI32<U3>]>> for HostAdd {
    type Output = HostCallResult<Store<ZeroPages>, tarr![WasmI32<U5>]>;
}

type ImportAddEnv = WasmHostEnv<
    tarr![HostFuncBinding<tstr::TS!("host"), tstr::TS!("add"), HostAdd>],
    TTerm,
    TTerm,
    TTerm
>;

#[test]
fn imported_function_calls_resolve_through_host_env() {
    type Final = InvokeFuncWithEnv<ImportAddModule, ImportAddEnv, U1, tarr![WasmI32<U2>, tarr![WasmI32<U3>]>>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
}
