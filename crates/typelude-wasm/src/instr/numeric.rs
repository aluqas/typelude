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
    opcode::{
        OpDrop, OpI32Add, OpI32And, OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32GeS, OpI32GeU,
        OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or,
        OpI32RemS, OpI32RemU, OpI32Shl, OpI32ShrS, OpI32ShrU, OpI32Sub, OpI32Xor,
    },
    run::Step,
    state::WasmState,
    value::WasmI32,
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
