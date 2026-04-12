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
    OpBlock, OpBr, OpBrIf, OpCall, OpCallIndirect, OpIf, OpLoop, OpReturn, OpSelect,
};
#[doc(hidden)]
pub use control::{OpEndBlock, OpEndFunc, OpEndLoop};
pub use global::{OpGlobalGet, OpGlobalSet};
pub use local::{OpLocalGet, OpLocalSet, OpLocalTee};
pub use memory::{
    OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8, OpI64Load, OpI64Store, OpMemoryGrow,
    OpMemorySize,
};
pub use numeric::{
    OpDrop, OpI32Add, OpI32And, OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32GeS, OpI32GeU,
    OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or,
    OpI32RemS, OpI32RemU, OpI32Shl, OpI32ShrS, OpI32ShrU, OpI32Sub, OpI32Xor, OpI64Add, OpI64And,
    OpI64DivS, OpI64DivU, OpI64Eq, OpI64Eqz, OpI64GeS, OpI64GeU, OpI64GtS, OpI64GtU, OpI64LeS,
    OpI64LeU, OpI64LtS, OpI64LtU, OpI64Mul, OpI64Ne, OpI64Or, OpI64RemS, OpI64RemU, OpI64Shl,
    OpI64ShrS, OpI64ShrU, OpI64Sub, OpI64Xor,
};
