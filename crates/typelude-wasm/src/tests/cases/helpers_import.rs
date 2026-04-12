#[test]
fn imported_global_compat_accepts_matching_i32_and_i64_globals() {
    type I32Compat = <WasmGlobal<GlobalConst, WasmI32<U1>> as ImportedGlobalCompat<
        ImportGlobal<GlobalConst, WasmI32Type>,
    >>::Output;
    type I64Compat = <WasmGlobal<GlobalMut, WasmI64<U2>> as ImportedGlobalCompat<
        ImportGlobal<GlobalMut, WasmI64Type>,
    >>::Output;

    assert_type_eq_all!(I32Compat, WasmGlobal<GlobalConst, WasmI32<U1>>);
    assert_type_eq_all!(I64Compat, WasmGlobal<GlobalMut, WasmI64<U2>>);
}
