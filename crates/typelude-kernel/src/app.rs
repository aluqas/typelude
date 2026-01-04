//! **Unified Application AST**

use std::marker::PhantomData;

use crate::{
    Eval, Evaluate, Apply,
};

/// **Generic Function Application**: `App<Op, Args>`
pub struct App<Op, Args>(PhantomData<(Op, Args)>);

impl<Op, Args> Eval for App<Op, Args>
where
    Op: Apply<Args>,
    Op::Output: Eval,
{
    type Output = Evaluate<Op::Output>;
}
