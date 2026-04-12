type GrowOneToTwoPagesModule = ModuleWithMemoryLimits<TTerm, U1, U2>;
type PreloadedDataModule = ModuleWithData<
    TTerm,
    U1,
    tarr![WasmDataSegment<WasmConstExpr<tarr![OpI32Const<U2>]>, tarr![U7, U8]>]
>;

#[test]
fn memory_size_reports_initial_page_count() {
    type Program = tarr![OpMemorySize];
    type Final = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U1>]);
}

#[test]
fn memory_grow_within_max_returns_old_page_count_and_updates_memory() {
    type Program = tarr![OpI32Const<U1>, OpMemoryGrow, OpMemorySize];
    type Final = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>, WasmI32<U1>]);
}

#[test]
fn store_then_load_round_trips_i32_value() {
    type Program = tarr![
        OpI32Const<U0>,
        OpI32Const<U258>,
        OpI32Store<U0>,
        OpI32Const<U0>,
        OpI32Load<U0>
    ];
    type Final = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U258>]);
}

#[test]
fn active_data_segment_preloads_memory_before_first_instruction() {
    type Program = tarr![OpI32Const<U2>, OpI32Load8U<U0>];
    type Final = ModuleProgramRun<PreloadedDataModule, Program>;

    assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U7>]);
}
