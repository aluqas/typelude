#[cfg(test)]
mod debug_tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::ELit;
    use typelude_std::tyarray;
    use typenum::{B0, B1, U0, U3};

    use super::*;
    use crate::machine::instruction::{OpDup, OpGt, OpPush};

    #[test]
    fn test_op_gt_behavior() {
        // Stack: [3].
        // Dup -> [3, 3]
        // Push 0 -> [0, 3, 3]
        // Gt -> [3 > 0, 3]
        type Prog = tyarray![OpPush<ELit<U3>>, OpDup, OpPush<ELit<U0>>, OpGt];

        type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        // Expect: [True, 3] i.e. [B1, 3]
        // If OpGt is correct (3 > 0), Head should be B1.
        type Head = typelude_std::std::col::array::Head<FinalStack>;

        // Assert Head is ELit<B1> (True)
        assert_type_eq_all!(Head, ELit<B1>);
    }
}
