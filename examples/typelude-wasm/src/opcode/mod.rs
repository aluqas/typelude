//! WASM スタックマシン命令セット。
//!
//! i32/i64 数値、制御フロー、メモリ、ローカル変数、グローバル変数の
//! アクセス命令をマーカー型で定義。各命令は Op トレイトを通じて
//! スタック状態変換を記述できる。

mod constant;
mod control;
mod global;
mod local;
mod memory;
mod numeric;

pub use constant::{OpI32Const, OpI64Const};
pub use control::{
    OpBlock, OpBr, OpBrIf, OpBrTable, OpCall, OpCallIndirect, OpIf, OpLoop, OpNop, OpReturn,
    OpSelect, OpUnreachable,
};
#[doc(hidden)]
pub use control::{OpEndBlock, OpEndFunc, OpEndLoop};
pub use global::{OpGlobalGet, OpGlobalSet};
pub use local::{OpLocalGet, OpLocalSet, OpLocalTee};
pub use memory::{
    OpI32Load, OpI32Load8S, OpI32Load8U, OpI32Load16S, OpI32Load16U, OpI32Store, OpI32Store8,
    OpI32Store16, OpI64Load, OpI64Load8S, OpI64Load8U, OpI64Load16S, OpI64Load16U, OpI64Load32S,
    OpI64Load32U, OpI64Store, OpI64Store8, OpI64Store16, OpI64Store32, OpMemoryGrow, OpMemorySize,
};
pub use numeric::{
    OpDrop, OpF32ReinterpretI32, OpF64ReinterpretI64, OpI32Add, OpI32And, OpI32Clz, OpI32Ctz,
    OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32Extend8S, OpI32Extend16S, OpI32GeS, OpI32GeU,
    OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or,
    OpI32Popcnt, OpI32RemS, OpI32RemU, OpI32Rotl, OpI32Rotr, OpI32Shl, OpI32ShrS, OpI32ShrU,
    OpI32Sub, OpI32WrapI64, OpI32Xor, OpI64Add, OpI64And, OpI64Clz, OpI64Ctz, OpI64DivS,
    OpI64DivU, OpI64Eq, OpI64Eqz, OpI64ExtendI32S, OpI64ExtendI32U, OpI64GeS, OpI64GeU, OpI64GtS,
    OpI64GtU, OpI64LeS, OpI64LeU, OpI64LtS, OpI64LtU, OpI64Mul, OpI64Ne, OpI64Or, OpI64Popcnt,
    OpI64ReinterpretF64, OpI64RemS, OpI64RemU, OpI64Rotl, OpI64Rotr, OpI64Shl, OpI64ShrS,
    OpI64ShrU, OpI64Sub, OpI64Xor,
};
