#[test]
fn i64_wrapping_add_and_sub_follow_64bit_bitpatterns() {
    type AddProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64Add];
    type AddFinal = ModuleProgramRun<EmptyModule, AddProgram>;
    type SubProgram = tarr![OpI64Const<U0>, OpI64Const<U1>, OpI64Sub];
    type SubFinal = ModuleProgramRun<EmptyModule, SubProgram>;

    assert_type_eq_all!(<AddFinal as StateStack>::Output, tarr![WasmI64<U0>]);
    assert_type_eq_all!(<SubFinal as StateStack>::Output, tarr![WasmI64<U18446744073709551615>]);
}

#[test]
fn i64_signed_unsigned_compare_and_shift_behave_correctly() {
    type SignedLtProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64LtS];
    type SignedLtFinal = ModuleProgramRun<EmptyModule, SignedLtProgram>;
    type UnsignedGtProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64GtU];
    type UnsignedGtFinal = ModuleProgramRun<EmptyModule, UnsignedGtProgram>;
    type ShrSProgram = tarr![OpI64Const<U18446744073709551614>, OpI64Const<U1>, OpI64ShrS];
    type ShrSFinal = ModuleProgramRun<EmptyModule, ShrSProgram>;

    assert_type_eq_all!(<SignedLtFinal as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<UnsignedGtFinal as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<ShrSFinal as StateStack>::Output, tarr![WasmI64<U18446744073709551615>]);
}
