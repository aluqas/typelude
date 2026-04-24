use typelude_col::{TArr, TTerm};
use typelude_std::core::Concat;

use crate::opcode::control::{OpIf, OpWhile};

pub trait LowerWhile<Rest> {
    type Output;
}

impl<CondProg, BodyProg, Rest> LowerWhile<Rest> for OpWhile<CondProg, BodyProg>
where
    BodyProg: Concat<TArr<OpWhile<CondProg, BodyProg>, TTerm>>,
    CondProg: Concat<
        TArr<
            OpIf<<BodyProg as Concat<TArr<OpWhile<CondProg, BodyProg>, TTerm>>>::Output, TTerm>,
            Rest,
        >,
    >,
{
    type Output = <CondProg as Concat<
        TArr<
            OpIf<<BodyProg as Concat<TArr<OpWhile<CondProg, BodyProg>, TTerm>>>::Output, TTerm>,
            Rest,
        >,
    >>::Output;
}
