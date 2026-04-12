type ExportStart = Fn0<TTerm, tarr![OpI32Const<U7>, OpGlobalSet<U0>, OpReturn]>;
type ExportMain = Fn0<TTerm, tarr![OpGlobalGet<U0>, OpReturn]>;
type ExportedStateModule = WasmModule<
    TTerm,
    WasmFuncSpace<
        tarr![WasmFuncType<TTerm, TTerm>, WasmFuncType<TTerm, tarr![WasmI32Type]>],
        tarr![ExportStart, ExportMain]
    >,
    WasmModuleMemory<WasmMemoryDecl<U1, NoLimit>, TTerm>,
    WasmModuleTables<tarr![WasmTableDecl<U1, U1>], tarr![WasmElemSegment<U0, WasmConstExpr<tarr![OpI32Const<U0>]>, tarr![U1]>]>,
    tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U2>>],
    tarr![
        WasmExport<tstr::TS!("main"), ExportFunc<U1>>,
        WasmExport<tstr::TS!("counter"), ExportGlobal<U0>>,
        WasmExport<tstr::TS!("mem"), ExportMemory>,
        WasmExport<tstr::TS!("table"), ExportTable<U0>>
    ],
    StartFunc<U0>
>;

#[test]
fn export_accessors_read_live_state_entries_by_name() {
    type Final = ModuleProgramRun<ExportedStateModule, tarr![OpCall<U1>]>;

    assert_type_eq_all!(<Final as StateExportGlobal<tstr::TS!("counter")>>::Output, WasmGlobal<GlobalMut, WasmI32<U7>>);
    assert_type_eq_all!(<Final as StateExportMemory<tstr::TS!("mem")>>::Output, WasmMemory<U1, MaxPages, TTerm>);
    assert_type_eq_all!(<Final as StateExportTable<tstr::TS!("table")>>::Output, WasmTable<U1, U1, tarr![TableEntry<U0, U1>]>);
}
