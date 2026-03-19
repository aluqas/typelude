use core::marker::PhantomData;

use typelude_std::{core::Eval, std::col::array::Nil};

#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames, Labels = Nil, Program = Nil>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Labels, Program)>,
);

impl<S, L, M, F, B, P> Eval for VmState<S, L, M, F, B, P> {
    type Output = Self;
}
