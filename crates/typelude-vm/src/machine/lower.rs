use typelude_std::std::col::array::{Array, Concat, IsList, Nil};

use crate::machine::instr::core::{OpIf, OpWhile};

pub trait LowerInstr<Rest: IsList> {
    type Output: IsList;
}

impl<CondProg, BodyProg, Rest> LowerInstr<Rest> for OpWhile<CondProg, BodyProg>
where
    Rest: IsList,
    BodyProg: Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>,
    <BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output: IsList,
    CondProg: Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >,
    <CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >>::Output: Concat<Rest>,
{
    type Output = <<CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >>::Output as Concat<Rest>>::Output;
}
