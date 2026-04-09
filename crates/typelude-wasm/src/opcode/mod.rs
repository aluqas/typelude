mod constant;
mod control;
mod local;
mod memory;
mod numeric;

pub use constant::OpI32Const;
pub use control::{OpBlock, OpBr, OpBrIf, OpCall, OpIf, OpLoop, OpReturn, OpSelect};
#[doc(hidden)]
pub use control::{OpEndBlock, OpEndFunc, OpEndLoop};
pub use local::{OpLocalGet, OpLocalSet, OpLocalTee};
pub use memory::{OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8, OpMemorySize};
pub use numeric::{OpI32Add, OpI32Eqz, OpI32Sub};
