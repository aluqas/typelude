use typelude_col::TArr;
use typelude_std::core::{Add, Eq, Eval, Sub};
use typenum::{B0, B1, U0, U1};

use crate::{
    opcode::{OpI32Add, OpI32Eqz, OpI32Sub},
    run::Step,
    state::WasmState,
    value::WasmI32,
};

#[doc(hidden)]
pub trait EqzValue {
    type Output;
}

impl EqzValue for B0 {
    type Output = WasmI32<U0>;
}

impl EqzValue for B1 {
    type Output = WasmI32<U1>;
}

impl<Module, Lhs, Rhs, Tail, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpI32Add, Rest>,
        >,
    >
where
    Lhs: Add<Rhs>,
{
    type Output = WasmState<
        Module,
        TArr<WasmI32<<Lhs as Add<Rhs>>::Output>, Tail>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Lhs, Rhs, Tail, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Rhs>, TArr<WasmI32<Lhs>, Tail>>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpI32Sub, Rest>,
        >,
    >
where
    Lhs: Sub<Rhs>,
{
    type Output = WasmState<
        Module,
        TArr<WasmI32<<Lhs as Sub<Rhs>>::Output>, Tail>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Value, Tail, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Value>, Tail>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpI32Eqz, Rest>,
        >,
    >
where
    Value: Eq<U0>,
    <Value as Eq<U0>>::Output: EqzValue,
{
    type Output = WasmState<
        Module,
        TArr<<<Value as Eq<U0>>::Output as EqzValue>::Output, Tail>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}
