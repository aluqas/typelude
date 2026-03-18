use core::marker::PhantomData;

use typelude_std::core::Eval;

pub use crate::shared::frame::{CallFrame, LabelFrame};

#[derive(Debug)]
pub struct CoreState<Stack, Locals, Memory, Frames, Labels, Program>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Labels, Program)>,
);

impl<S, L, M, F, B, P> Eval for CoreState<S, L, M, F, B, P> {
    type Output = Self;
}
