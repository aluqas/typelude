pub use crate::vm::effect::{BeforeStep as ConsumeFuel, trace::EmitTrace};

pub trait WriteLog<Item> {
    type Output;
}

pub trait UseWorld<World> {
    type Output;
}
