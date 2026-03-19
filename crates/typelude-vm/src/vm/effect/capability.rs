pub use crate::vm::effect::BeforeStep as ConsumeFuel;
pub use crate::vm::effect::trace::EmitTrace;

pub trait WriteLog<Item> {
    type Output;
}

pub trait UseWorld<World> {
    type Output;
}
