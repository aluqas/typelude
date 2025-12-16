use typelude::{machine::execution::Execute, prelude::*, program};

fn main() {
    // Attempt to pop from an empty stack.
    // This should fail to compile because the Stack (TyNil) does not implement the necessary traits.
    type Program = program! {
        (drop) // Pop from empty stack
    };

    // Standard runner setup
    type InitialState = typelude::machine::state::MachineState<
        typelude::std::array::TyNil, // Stack
        typelude::std::array::TyNil, // Locals
        typelude::std::array::TyNil, // Memory
        typelude::std::array::TyNil, // CallStack
        Program,
    >;

    // Attempt evaluation. This should fail because OpDrop cannot execute on TyNil stack.
    type FinalState = typelude::eval::Evaluator<typelude::machine::execution::ERun<InitialState>>;

    // We don't need to instantiate it, just naming the type triggers the check.
    let _ = std::marker::PhantomData::<FinalState>;
}
