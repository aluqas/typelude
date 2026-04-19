//! checked runtime で既存 `Step` をそのまま再利用できる opcode 群。
//!
//! checked runtime は trap-aware な opcode だけを個別実装し、それ以外は
//! success-only `Step` の結果を `WasmDone` で包みます。このファイルの
//! `InfallibleOpcode` 実装一覧がその fallback 対象を定義します。

use typelude_col::TArr;
use typelude_std::core::{Eval, Evaluate};
use typenum::U0;

use crate::{
    opcode::{
        OpBlock, OpBr, OpBrIf, OpBrTable, OpCall, OpDrop, OpEndBlock, OpEndFunc, OpEndLoop,
        OpF32ReinterpretI32, OpF64ReinterpretI64, OpGlobalGet, OpGlobalSet, OpI32Add, OpI32And,
        OpI32Clz, OpI32Const, OpI32Ctz, OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32Extend8S,
        OpI32Extend16S, OpI32GeS, OpI32GeU, OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS,
        OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or, OpI32Popcnt, OpI32RemS, OpI32RemU, OpI32Rotl,
        OpI32Rotr, OpI32Shl, OpI32ShrS, OpI32ShrU, OpI32Sub, OpI32WrapI64, OpI32Xor, OpI64Add,
        OpI64And, OpI64Clz, OpI64Const, OpI64Ctz, OpI64DivS, OpI64DivU, OpI64Eq, OpI64Eqz,
        OpI64ExtendI32S, OpI64ExtendI32U, OpI64GeS, OpI64GeU, OpI64GtS, OpI64GtU, OpI64LeS,
        OpI64LeU, OpI64LtS, OpI64LtU, OpI64Mul, OpI64Ne, OpI64Or, OpI64Popcnt,
        OpI64ReinterpretF64, OpI64RemS, OpI64RemU, OpI64Rotl, OpI64Rotr, OpI64Shl, OpI64ShrS,
        OpI64ShrU, OpI64Sub, OpI64Xor, OpIf, OpLocalGet, OpLocalSet, OpLocalTee, OpLoop,
        OpMemoryGrow, OpMemorySize, OpNop, OpReturn, OpSelect,
    },
    run::{CheckedStep, InfallibleOpcode, Step, WasmDone},
    state::WasmState,
};

macro_rules! impl_infallible_unit {
    ($($op:ty),* $(,)?) => {
        $(
            impl InfallibleOpcode for $op {}
        )*
    };
}

macro_rules! impl_infallible_generic {
    ($($op:ident<$($param:ident),+>),* $(,)?) => {
        $(
            impl<$($param),+> InfallibleOpcode for $op<$($param),+> {}
        )*
    };
}

impl_infallible_unit!(
    OpDrop,
    OpNop,
    OpI32Add,
    OpI32And,
    OpI32Clz,
    OpI32Sub,
    OpI32Ctz,
    OpI32DivS,
    OpI32DivU,
    OpI32Eq,
    OpI32Eqz,
    OpI32Extend8S,
    OpI32Extend16S,
    OpI32GeS,
    OpI32GeU,
    OpI32GtS,
    OpI32GtU,
    OpI32LeS,
    OpI32LeU,
    OpI32LtS,
    OpI32LtU,
    OpI32Mul,
    OpI32Ne,
    OpI32Or,
    OpI32Popcnt,
    OpI32RemS,
    OpI32RemU,
    OpI32Rotl,
    OpI32Rotr,
    OpI32Shl,
    OpI32ShrS,
    OpI32ShrU,
    OpI32WrapI64,
    OpI32Xor,
    OpF32ReinterpretI32,
    OpF64ReinterpretI64,
    OpI64Add,
    OpI64And,
    OpI64Clz,
    OpI64Sub,
    OpI64Ctz,
    OpI64Eqz,
    OpI64Eq,
    OpI64ExtendI32S,
    OpI64ExtendI32U,
    OpI64Ne,
    OpI64LtS,
    OpI64LtU,
    OpI64GtS,
    OpI64GtU,
    OpI64LeS,
    OpI64LeU,
    OpI64GeS,
    OpI64GeU,
    OpI64Or,
    OpI64Popcnt,
    OpI64Xor,
    OpI64Shl,
    OpI64ShrS,
    OpI64ShrU,
    OpI64Rotl,
    OpI64Rotr,
    OpI64Mul,
    OpI64DivS,
    OpI64DivU,
    OpI64RemS,
    OpI64RemU,
    OpI64ReinterpretF64,
    OpReturn,
    OpSelect,
    OpEndFunc,
    OpEndBlock,
    OpEndLoop
);

impl_infallible_generic!(
    OpI32Const<Val>,
    OpI64Const<Val>,
    OpLocalGet<Idx>,
    OpLocalSet<Idx>,
    OpLocalTee<Idx>,
    OpGlobalGet<Idx>,
    OpGlobalSet<Idx>,
    OpBlock<Body>,
    OpLoop<Body>,
    OpBr<Depth>,
    OpBrIf<Depth>,
    OpBrTable<Targets, Default>,
    OpIf<Then, Else>,
    OpCall<FuncIdx>
);

impl InfallibleOpcode for OpMemorySize<U0> {}
impl InfallibleOpcode for OpMemoryGrow<U0> {}

impl<Module, Store, Stack, Locals, Frames, Branches, Instr, Rest> Eval
    for CheckedStep<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>
where
    Instr: InfallibleOpcode,
    Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>: Eval,
{
    type Output = WasmDone<
        Evaluate<
            Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>,
        >,
    >;
}
