#![recursion_limit = "1024"]

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::{ELit, Evaluate};
    use typelude_std::{
        lambda::{LApp, church},
        std::col::array::Nil,
        tyarray,
    };
    use typelude_vm::machine::{
        compiler::{Compile, ToAction},
        instruction::{OpAdd, OpPush},
        semantics::{monad::Unit, state::VmState},
    };
    use typenum::{U1, U2, U3};
    // Define aliases for clarity
    type EmptyState = VmState<Nil, Nil, Nil, Nil>;
    type Stack<S> = VmState<S, Nil, Nil, Nil>;

    #[test]
    fn test_compile_push_push() {
        // Program: Push 1, Push 2
        type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>];

        // Compile
        type CompiledAction = <Prog as Compile>::Output;

        // Run
        // type ResultPair = Evaluate<LApp<CompiledAction, EmptyState>>;
        type ResultPair = <CompiledAction as typelude_core::Apply<EmptyState>>::Output;

        // Expected Result: LPair2<Unit, Stack<[2, 1]>>
        type ExpectedStack = tyarray![U2, U1];
        type ExpectedState = Stack<ExpectedStack>;
        type ExpectedResult = typelude_std::lambda::church::LPair2<Unit, ExpectedState>;

        assert_type_eq_all!(ResultPair, ExpectedResult);
    }

    #[test]
    fn test_compile_push_add() {
        // Program: Push 1, Push 2, Add
        type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];

        // Compile
        type CompiledAction = <Prog as Compile>::Output;

        // Run: Apply CompiledAction to EmptyState
        // Result is (Unit, FinalState)
        // type ResultPair = Evaluate<LApp<CompiledAction, EmptyState>>;
        type ResultPair = <CompiledAction as typelude_core::Apply<EmptyState>>::Output;

        // Expected Result: LPair2<Unit, Stack<[3]>>
        type ExpectedStack = tyarray![U3];
        type ExpectedState = Stack<ExpectedStack>;
        type ExpectedResult = typelude_std::lambda::church::LPair2<Unit, ExpectedState>;

        assert_type_eq_all!(ResultPair, ExpectedResult);
    }
}
