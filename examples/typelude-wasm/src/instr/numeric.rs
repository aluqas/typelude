use typelude_col::TArr;
use typelude_std::core::{Eq, Eval};
use typenum::U0;

use crate::{
    helpers::{
        i32::{
            BoolNot, BoolToI32, I32Add, I32AndValue, I32Clz, I32Ctz, I32DivS, I32DivU, I32EqValue,
            I32Extend8S, I32Extend16S, I32Mul, I32OrValue, I32Popcnt, I32RemS, I32RemU,
            I32RotlValue, I32RotrValue, I32ShlValue, I32ShrSValue, I32ShrUValue, I32SignedGe,
            I32SignedGt, I32SignedLe, I32SignedLt, I32Sub, I32UnsignedGe, I32UnsignedGt,
            I32UnsignedLe, I32UnsignedLt, I32XorValue,
        },
        i64::{
            I32WrapI64, I64Add, I64AndValue, I64Clz, I64Ctz, I64DivS, I64DivU, I64EqValue,
            I64ExtendI32S, I64ExtendI32U, I64Mul, I64OrValue, I64Popcnt, I64RemS, I64RemU,
            I64RotlValue, I64RotrValue, I64ShlValue, I64ShrSValue, I64ShrUValue, I64SignedGe,
            I64SignedGt, I64SignedLe, I64SignedLt, I64Sub, I64UnsignedGe, I64UnsignedGt,
            I64UnsignedLe, I64UnsignedLt, I64XorValue,
        },
    },
    opcode::{
        OpDrop, OpF32ReinterpretI32, OpF64ReinterpretI64, OpI32Add, OpI32And, OpI32Clz, OpI32Ctz,
        OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32Extend8S, OpI32Extend16S, OpI32GeS,
        OpI32GeU, OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne,
        OpI32Or, OpI32Popcnt, OpI32RemS, OpI32RemU, OpI32Rotl, OpI32Rotr, OpI32Shl, OpI32ShrS,
        OpI32ShrU, OpI32Sub, OpI32WrapI64, OpI32Xor, OpI64Add, OpI64And, OpI64Clz, OpI64Ctz,
        OpI64DivS, OpI64DivU, OpI64Eq, OpI64Eqz, OpI64ExtendI32S, OpI64ExtendI32U, OpI64GeS,
        OpI64GeU, OpI64GtS, OpI64GtU, OpI64LeS, OpI64LeU, OpI64LtS, OpI64LtU, OpI64Mul, OpI64Ne,
        OpI64Or, OpI64Popcnt, OpI64ReinterpretF64, OpI64RemS, OpI64RemU, OpI64Rotl, OpI64Rotr,
        OpI64Shl, OpI64ShrS, OpI64ShrU, OpI64Sub, OpI64Xor,
    },
    run::Step,
    state::WasmState,
    value::{WasmF32, WasmF64, WasmI32, WasmI64},
};

#[doc(hidden)]
/// `B0` / `B1` を Wasm の真偽値表現 `WasmI32<0|1>` へ変換する helper。
pub trait BoolResult {
    type Output;
}

impl BoolResult for typenum::B0 {
    type Output = WasmI32<U0>;
}

impl BoolResult for typenum::B1 {
    type Output = WasmI32<typenum::U1>;
}

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, TArr<ValueT, Tail>, Locals, Frames, Branches, TArr<OpDrop, Rest>>,
    >
{
    type Output = WasmState<Module, Store, Tail, Locals, Frames, Branches, Rest>;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Add, Rest>,
        >,
    >
where
    Lhs: I32Add<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI32<<Lhs as I32Add<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Sub, Rest>,
        >,
    >
where
    Lhs: I32Sub<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI32<<Lhs as I32Sub<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Mul, Rest>,
        >,
    >
where
    Lhs: I32Mul<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI32<<Lhs as I32Mul<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Eqz, Rest>,
        >,
    >
where
    ValueT: Eq<U0>,
    <ValueT as Eq<U0>>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<ValueT as Eq<U0>>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32WrapI64, Rest>,
        >,
    >
where
    ValueT: I32WrapI64,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI32<<ValueT as I32WrapI64>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Bits, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Bits>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpF32ReinterpretI32, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmF32<Bits>, Tail>, Locals, Frames, Branches, Rest>;
}

impl<Module, Store, Bits, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Bits>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpF64ReinterpretI64, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmF64<Bits>, Tail>, Locals, Frames, Branches, Rest>;
}

impl<Module, Store, Bits, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmF64<Bits>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64ReinterpretF64, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, TArr<WasmI64<Bits>, Tail>, Locals, Frames, Branches, Rest>;
}

macro_rules! impl_i32_unary_value_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI32<ValueT>, Tail>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            ValueT: $trait,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<WasmI32<<ValueT as $trait>::Output>, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_i32_unary_value_op!(OpI32Clz, I32Clz);
impl_i32_unary_value_op!(OpI32Ctz, I32Ctz);
impl_i32_unary_value_op!(OpI32Popcnt, I32Popcnt);
impl_i32_unary_value_op!(OpI32Extend8S, I32Extend8S);
impl_i32_unary_value_op!(OpI32Extend16S, I32Extend16S);

macro_rules! impl_cmp_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            Lhs: $trait<Rhs>,
            <Lhs as $trait<Rhs>>::Output: BoolResult,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<<<Lhs as $trait<Rhs>>::Output as BoolResult>::Output, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_cmp_op!(OpI32LtS, I32SignedLt);
impl_cmp_op!(OpI32LtU, I32UnsignedLt);
impl_cmp_op!(OpI32GtS, I32SignedGt);
impl_cmp_op!(OpI32GtU, I32UnsignedGt);
impl_cmp_op!(OpI32LeS, I32SignedLe);
impl_cmp_op!(OpI32LeU, I32UnsignedLe);
impl_cmp_op!(OpI32GeS, I32SignedGe);
impl_cmp_op!(OpI32GeU, I32UnsignedGe);

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Eq, Rest>,
        >,
    >
where
    Lhs: I32EqValue<Rhs>,
    <Lhs as I32EqValue<Rhs>>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<Lhs as I32EqValue<Rhs>>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Ne, Rest>,
        >,
    >
where
    Lhs: I32EqValue<Rhs>,
    <Lhs as I32EqValue<Rhs>>::Output: BoolNot,
    <<Lhs as I32EqValue<Rhs>>::Output as BoolNot>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<<Lhs as I32EqValue<Rhs>>::Output as BoolNot>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

macro_rules! impl_bin_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            Lhs: $trait<Rhs>,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<WasmI32<<Lhs as $trait<Rhs>>::Output>, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_bin_op!(OpI32And, I32AndValue);
impl_bin_op!(OpI32Or, I32OrValue);
impl_bin_op!(OpI32Xor, I32XorValue);
impl_bin_op!(OpI32Shl, I32ShlValue);
impl_bin_op!(OpI32ShrS, I32ShrSValue);
impl_bin_op!(OpI32ShrU, I32ShrUValue);
impl_bin_op!(OpI32Rotl, I32RotlValue);
impl_bin_op!(OpI32Rotr, I32RotrValue);
impl_bin_op!(OpI32DivS, I32DivS);
impl_bin_op!(OpI32DivU, I32DivU);
impl_bin_op!(OpI32RemS, I32RemS);
impl_bin_op!(OpI32RemU, I32RemU);

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Add, Rest>,
        >,
    >
where
    Lhs: I64Add<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI64<<Lhs as I64Add<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

macro_rules! impl_i64_unary_value_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI64<ValueT>, Tail>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            ValueT: $trait,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<WasmI64<<ValueT as $trait>::Output>, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_i64_unary_value_op!(OpI64Clz, I64Clz);
impl_i64_unary_value_op!(OpI64Ctz, I64Ctz);
impl_i64_unary_value_op!(OpI64Popcnt, I64Popcnt);

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Sub, Rest>,
        >,
    >
where
    Lhs: I64Sub<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI64<<Lhs as I64Sub<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Mul, Rest>,
        >,
    >
where
    Lhs: I64Mul<Rhs>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI64<<Lhs as I64Mul<Rhs>>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Eqz, Rest>,
        >,
    >
where
    ValueT: Eq<U0>,
    <ValueT as Eq<U0>>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<ValueT as Eq<U0>>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

macro_rules! impl_i64_cmp_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            Lhs: $trait<Rhs>,
            <Lhs as $trait<Rhs>>::Output: BoolResult,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<<<Lhs as $trait<Rhs>>::Output as BoolResult>::Output, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_i64_cmp_op!(OpI64LtS, I64SignedLt);
impl_i64_cmp_op!(OpI64LtU, I64UnsignedLt);
impl_i64_cmp_op!(OpI64GtS, I64SignedGt);
impl_i64_cmp_op!(OpI64GtU, I64UnsignedGt);
impl_i64_cmp_op!(OpI64LeS, I64SignedLe);
impl_i64_cmp_op!(OpI64LeU, I64UnsignedLe);
impl_i64_cmp_op!(OpI64GeS, I64SignedGe);
impl_i64_cmp_op!(OpI64GeU, I64UnsignedGe);

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Eq, Rest>,
        >,
    >
where
    Lhs: I64EqValue<Rhs>,
    <Lhs as I64EqValue<Rhs>>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<Lhs as I64EqValue<Rhs>>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Ne, Rest>,
        >,
    >
where
    Lhs: I64EqValue<Rhs>,
    <Lhs as I64EqValue<Rhs>>::Output: BoolNot,
    <<Lhs as I64EqValue<Rhs>>::Output as BoolNot>::Output: BoolResult,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<<Lhs as I64EqValue<Rhs>>::Output as BoolNot>::Output as BoolResult>::Output, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

macro_rules! impl_i64_bin_op {
    ($opcode:ty, $trait:ident) => {
        impl<Module, Store, Lhs, Rhs, Tail, Locals, Frames, Branches, Rest> Eval
            for Step<
                WasmState<
                    Module,
                    Store,
                    TArr<WasmI64<Rhs>, TArr<WasmI64<Lhs>, Tail>>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$opcode, Rest>,
                >,
            >
        where
            Lhs: $trait<Rhs>,
        {
            type Output = WasmState<
                Module,
                Store,
                TArr<WasmI64<<Lhs as $trait<Rhs>>::Output>, Tail>,
                Locals,
                Frames,
                Branches,
                Rest,
            >;
        }
    };
}

impl_i64_bin_op!(OpI64And, I64AndValue);
impl_i64_bin_op!(OpI64Or, I64OrValue);
impl_i64_bin_op!(OpI64Xor, I64XorValue);
impl_i64_bin_op!(OpI64Shl, I64ShlValue);
impl_i64_bin_op!(OpI64ShrS, I64ShrSValue);
impl_i64_bin_op!(OpI64ShrU, I64ShrUValue);
impl_i64_bin_op!(OpI64Rotl, I64RotlValue);
impl_i64_bin_op!(OpI64Rotr, I64RotrValue);
impl_i64_bin_op!(OpI64DivS, I64DivS);
impl_i64_bin_op!(OpI64DivU, I64DivU);
impl_i64_bin_op!(OpI64RemS, I64RemS);
impl_i64_bin_op!(OpI64RemU, I64RemU);

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64ExtendI32S, Rest>,
        >,
    >
where
    ValueT: I64ExtendI32S,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI64<<ValueT as I64ExtendI32S>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Store, ValueT, Tail, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64ExtendI32U, Rest>,
        >,
    >
where
    ValueT: I64ExtendI32U,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<WasmI64<<ValueT as I64ExtendI32U>::Output>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    #[test]
    fn i64_wrapping_add_and_sub_follow_64bit_bitpatterns() {
        type AddProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64Add];
        type AddFinal = ModuleProgramRun<EmptyModule, AddProgram>;
        type SubProgram = tarr![OpI64Const<U0>, OpI64Const<U1>, OpI64Sub];
        type SubFinal = ModuleProgramRun<EmptyModule, SubProgram>;

        assert_type_eq_all!(<AddFinal as StateStack>::Output, tarr![WasmI64<U0>]);
        assert_type_eq_all!(<SubFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709551615>
        ]);
    }

    #[test]
    fn i64_signed_unsigned_compare_and_shift_behave_correctly() {
        type SignedLtProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64LtS];
        type SignedLtFinal = ModuleProgramRun<EmptyModule, SignedLtProgram>;
        type UnsignedGtProgram =
            tarr![OpI64Const<U18446744073709551615>, OpI64Const<U1>, OpI64GtU];
        type UnsignedGtFinal = ModuleProgramRun<EmptyModule, UnsignedGtProgram>;
        type ShrSProgram = tarr![OpI64Const<U18446744073709551614>, OpI64Const<U1>, OpI64ShrS];
        type ShrSFinal = ModuleProgramRun<EmptyModule, ShrSProgram>;

        assert_type_eq_all!(<SignedLtFinal as StateStack>::Output, tarr![WasmI32<U1>]);
        assert_type_eq_all!(<UnsignedGtFinal as StateStack>::Output, tarr![WasmI32<U1>]);
        assert_type_eq_all!(<ShrSFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709551615>
        ]);
    }

    #[test]
    fn i32_unary_intrinsics_and_rotation_work() {
        type U128T = <typenum::Const<128> as typenum::ToUInt>::Output;
        type U4294967168 = <typenum::Const<4294967168usize> as typenum::ToUInt>::Output;
        type U32768T = <typenum::Const<32768> as typenum::ToUInt>::Output;
        type U4294934528 = <typenum::Const<4294934528usize> as typenum::ToUInt>::Output;
        type ClzProgram = tarr![OpI32Const<U0>, OpI32Clz];
        type ClzFinal = ModuleProgramRun<EmptyModule, ClzProgram>;
        type CtzProgram = tarr![OpI32Const<U8>, OpI32Ctz];
        type CtzFinal = ModuleProgramRun<EmptyModule, CtzProgram>;
        type PopcntProgram = tarr![OpI32Const<crate::tests::support::MaxPages>, OpI32Popcnt];
        type PopcntFinal = ModuleProgramRun<EmptyModule, PopcntProgram>;
        type RotlProgram = tarr![OpI32Const<U1>, OpI32Const<U1>, OpI32Rotl];
        type RotlFinal = ModuleProgramRun<EmptyModule, RotlProgram>;
        type RotrProgram = tarr![OpI32Const<U1>, OpI32Const<typenum::U33>, OpI32Rotr];
        type RotrFinal = ModuleProgramRun<EmptyModule, RotrProgram>;
        type Extend8Program = tarr![OpI32Const<U128T>, OpI32Extend8S];
        type Extend8Final = ModuleProgramRun<EmptyModule, Extend8Program>;
        type Extend16Program = tarr![OpI32Const<U32768T>, OpI32Extend16S];
        type Extend16Final = ModuleProgramRun<EmptyModule, Extend16Program>;

        assert_type_eq_all!(<ClzFinal as StateStack>::Output, tarr![
            WasmI32<crate::helpers::i32::U32>
        ]);
        assert_type_eq_all!(<CtzFinal as StateStack>::Output, tarr![WasmI32<U3>]);
        assert_type_eq_all!(<PopcntFinal as StateStack>::Output, tarr![
            WasmI32<crate::helpers::i32::U32>
        ]);
        assert_type_eq_all!(<RotlFinal as StateStack>::Output, tarr![WasmI32<U2>]);
        assert_type_eq_all!(<RotrFinal as StateStack>::Output, tarr![
            WasmI32<typenum::U2147483648>
        ]);
        assert_type_eq_all!(<Extend8Final as StateStack>::Output, tarr![WasmI32<U4294967168>]);
        assert_type_eq_all!(<Extend16Final as StateStack>::Output, tarr![WasmI32<U4294934528>]);
    }

    #[test]
    fn i64_unary_intrinsics_rotation_and_extend_work() {
        type U2147483648T = <typenum::Const<2147483648usize> as typenum::ToUInt>::Output;
        type U18446744071562067968 = typenum::operator_aliases::Diff<
            crate::helpers::i64::U18446744073709551615,
            typenum::operator_aliases::Diff<U2147483648T, typenum::U1>,
        >;
        type ClzProgram = tarr![OpI64Const<U0>, OpI64Clz];
        type ClzFinal = ModuleProgramRun<EmptyModule, ClzProgram>;
        type CtzProgram = tarr![OpI64Const<U8>, OpI64Ctz];
        type CtzFinal = ModuleProgramRun<EmptyModule, CtzProgram>;
        type PopcntProgram = tarr![OpI64Const<U18446744073709551615>, OpI64Popcnt];
        type PopcntFinal = ModuleProgramRun<EmptyModule, PopcntProgram>;
        type RotrProgram = tarr![OpI64Const<U2>, OpI64Const<U1>, OpI64Rotr];
        type RotrFinal = ModuleProgramRun<EmptyModule, RotrProgram>;
        type RotlProgram = tarr![OpI64Const<U1>, OpI64Const<typenum::U65>, OpI64Rotl];
        type RotlFinal = ModuleProgramRun<EmptyModule, RotlProgram>;
        type ExtendSProgram = tarr![OpI32Const<U2147483648T>, OpI64ExtendI32S];
        type ExtendSFinal = ModuleProgramRun<EmptyModule, ExtendSProgram>;
        type ExtendUProgram = tarr![OpI32Const<typenum::U4294967295>, OpI64ExtendI32U];
        type ExtendUFinal = ModuleProgramRun<EmptyModule, ExtendUProgram>;

        assert_type_eq_all!(<ClzFinal as StateStack>::Output, tarr![
            WasmI64<crate::helpers::i64::U64>
        ]);
        assert_type_eq_all!(<CtzFinal as StateStack>::Output, tarr![WasmI64<U3>]);
        assert_type_eq_all!(<PopcntFinal as StateStack>::Output, tarr![
            WasmI64<crate::helpers::i64::U64>
        ]);
        assert_type_eq_all!(<RotrFinal as StateStack>::Output, tarr![WasmI64<U1>]);
        assert_type_eq_all!(<RotlFinal as StateStack>::Output, tarr![WasmI64<U2>]);
        assert_type_eq_all!(<ExtendSFinal as StateStack>::Output, tarr![
            WasmI64<U18446744071562067968>
        ]);
        assert_type_eq_all!(<ExtendUFinal as StateStack>::Output, tarr![
            WasmI64<typenum::U4294967295>
        ]);
    }

    #[test]
    fn wrap_and_reinterpret_ops_preserve_expected_bitpatterns() {
        type F32OneBits = crate::wasm_u32_bits_le!(0x00, 0x00, 0x80, 0x3F);
        type WrapProgram = tarr![OpI64Const<U18446744073709551615>, OpI32WrapI64];
        type WrapFinal = ModuleProgramRun<EmptyModule, WrapProgram>;
        type F32Program = tarr![OpI32Const<F32OneBits>, OpF32ReinterpretI32];
        type F32Final = ModuleProgramRun<EmptyModule, F32Program>;
        type F64Program = tarr![OpI64Const<U18446744073709551615>, OpF64ReinterpretI64];
        type F64Final = ModuleProgramRun<EmptyModule, F64Program>;
        type RoundTripProgram =
            tarr![OpI64Const<U18446744073709551615>, OpF64ReinterpretI64, OpI64ReinterpretF64];
        type RoundTripFinal = ModuleProgramRun<EmptyModule, RoundTripProgram>;

        assert_type_eq_all!(<WrapFinal as StateStack>::Output, tarr![
            WasmI32<typenum::U4294967295>
        ]);
        assert_type_eq_all!(<F32Final as StateStack>::Output, tarr![WasmF32<F32OneBits>]);
        assert_type_eq_all!(<F64Final as StateStack>::Output, tarr![
            WasmF64<U18446744073709551615>
        ]);
        assert_type_eq_all!(<RoundTripFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709551615>
        ]);
    }

    #[test]
    fn float_carriers_flow_through_direct_call() {
        type ReinterpretEcho = WasmFunc<
            WasmFuncType<tarr![WasmF64Type], tarr![WasmF64Type]>,
            TTerm,
            tarr![OpLocalGet<U0>, OpReturn],
        >;
        type FloatModule = Module<tarr![ReinterpretEcho], U0>;
        type Program = tarr![OpI64Const<U18446744073709551615>, OpF64ReinterpretI64, OpCall<U0>];
        type Final = ModuleProgramRun<FloatModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmF64<U18446744073709551615>]);
    }
}
