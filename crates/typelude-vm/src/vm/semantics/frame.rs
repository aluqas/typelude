//! Call/return frame semantics.

use core::marker::PhantomData;

use typelude_std::core::Eval;

/// Return continuation capturing caller locals and the remaining caller
/// program.
#[derive(Debug)]
pub struct ReturnFrame<ReturnLocals, ReturnProgram>(
    pub PhantomData<(ReturnLocals, ReturnProgram)>,
);

impl<L, P> Eval for ReturnFrame<L, P> {
    type Output = Self;
}
