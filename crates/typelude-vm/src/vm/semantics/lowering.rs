use typelude_std::std::col::array::{Array, Concat, IsList, Nil};

use crate::opcode::control::{OpIf, OpWhile};

pub trait LowerWhile<Rest: IsList> {
    type Output: IsList;
}

impl<CondProg, BodyProg, Rest> LowerWhile<Rest> for OpWhile<CondProg, BodyProg>
where
    Rest: IsList,
    BodyProg: Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>,
    <BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output: IsList,
    CondProg: Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Rest,
        >,
    >,
{
    type Output = <CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Rest,
        >,
    >>::Output;
}
