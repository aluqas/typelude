use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct ReturnFrame<ReturnLocals, ReturnProgram>(
    pub PhantomData<(ReturnLocals, ReturnProgram)>,
);

impl<L, P> Eval for ReturnFrame<L, P> {
    type Output = Self;
}
