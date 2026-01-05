use typelude::{machine::execution::Execute, prelude::*, program};

fn main() {
    // Attempt to pop from an empty stack.
    // This should fail to compile because the Stack (Nil) does not implement the necessary traits.
    type Program = program! {
        (drop) // Pop from empty stack
    };

    // Standard runner setup
    type InitialState = typelude::machine::state::MachineState<
        typelude::std::array::Nil, // Stack
        typelude::std::array::Nil, // Locals
        typelude::std::array::Nil, // Memory
        typelude::std::array::Nil, // CallStack
        Program,
    >;

    // Attempt evaluation. This should fail because OpDrop cannot execute on Nil stack.
    type FinalState = typelude::eval::Evaluate<typelude::machine::execution::ERun<InitialState>>;

    // We don't need to instantiate it, just naming the type triggers the check.
    let _ = std::marker::PhantomData::<FinalState>;
}
