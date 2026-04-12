use typelude_col::TArr;
use typelude_std::core::{Eq, Eval};
use typenum::U0;

use crate::{
    helpers::i32::{
        BoolNot, BoolToI32, I32Add, I32AndValue, I32DivS, I32DivU, I32EqValue, I32Mul, I32OrValue,
        I32RemS, I32RemU, I32ShlValue, I32ShrSValue, I32ShrUValue, I32SignedGe, I32SignedGt,
        I32SignedLe, I32SignedLt, I32Sub, I32UnsignedGe, I32UnsignedGt, I32UnsignedLe,
        I32UnsignedLt, I32XorValue,
    },
    helpers::i64::{
        I64Add, I64AndValue, I64DivS, I64DivU, I64EqValue, I64Mul, I64OrValue, I64RemS, I64RemU,
        I64ShlValue, I64ShrSValue, I64ShrUValue, I64SignedGe, I64SignedGt, I64SignedLe,
        I64SignedLt, I64Sub, I64UnsignedGe, I64UnsignedGt, I64UnsignedLe, I64UnsignedLt,
        I64XorValue,
    },
    opcode::{
        OpDrop, OpI32Add, OpI32And, OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32GeS, OpI32GeU,
        OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or,
        OpI32RemS, OpI32RemU, OpI32Shl, OpI32ShrS, OpI32ShrU, OpI32Sub, OpI32Xor, OpI64Add,
        OpI64And, OpI64DivS, OpI64DivU, OpI64Eq, OpI64Eqz, OpI64GeS, OpI64GeU, OpI64GtS,
        OpI64GtU, OpI64LeS, OpI64LeU, OpI64LtS, OpI64LtU, OpI64Mul, OpI64Ne, OpI64Or,
        OpI64RemS, OpI64RemU, OpI64Shl, OpI64ShrS, OpI64ShrU, OpI64Sub, OpI64Xor,
    },
    run::Step,
    state::WasmState,
    value::{WasmI32, WasmI64},
};

#[doc(hidden)]
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
        WasmState<
            Module,
            Store,
            TArr<ValueT, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpDrop, Rest>,
        >,
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
impl_i64_bin_op!(OpI64DivS, I64DivS);
impl_i64_bin_op!(OpI64DivU, I64DivU);
impl_i64_bin_op!(OpI64RemS, I64RemS);
impl_i64_bin_op!(OpI64RemU, I64RemU);

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    include!("../tests/cases/instr_numeric.rs");
}
