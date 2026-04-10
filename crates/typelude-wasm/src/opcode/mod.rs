mod constant;
mod control;
mod global;
mod local;
mod memory;
mod numeric;

pub use constant::OpI32Const;
pub use control::{OpBlock, OpBr, OpBrIf, OpCall, OpCallIndirect, OpIf, OpLoop, OpReturn, OpSelect};
#[doc(hidden)]
pub use control::{OpEndBlock, OpEndFunc, OpEndLoop};
pub use global::{OpGlobalGet, OpGlobalSet};
pub use local::{OpLocalGet, OpLocalSet, OpLocalTee};
pub use memory::{OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8, OpMemoryGrow, OpMemorySize};
pub use numeric::{
    OpDrop, OpI32Add, OpI32And, OpI32DivS, OpI32DivU, OpI32Eq, OpI32Eqz, OpI32GeS, OpI32GeU,
    OpI32GtS, OpI32GtU, OpI32LeS, OpI32LeU, OpI32LtS, OpI32LtU, OpI32Mul, OpI32Ne, OpI32Or,
    OpI32RemS, OpI32RemU, OpI32Shl, OpI32ShrS, OpI32ShrU, OpI32Sub, OpI32Xor,
};
