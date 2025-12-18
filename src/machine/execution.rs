//! **Type-Level Stack Machine Execution Logic**
//!
//! Implements machine instruction execution logic and main loop.

use typenum::Unsigned;

// Important: Ensure std::int is imported so Eval impls for Arithmetic are visible
#[allow(unused_imports)]
use crate::std::int;
use crate::{
    eval::{EApp, EIf, ELit, EWhile, Eval, Evaluate},
    kernel::traits::Apply,
    machine::{instruction::*, state::MachineState},
    std::{
        array::{Cons, EConcat, Get, Set, TyArray, TyNil}, // OpIsEmpty unused
        bool::OpNot,
        // traits::Apply is in kernel
    },
};

//
// Execute Trait
//

/// Trait to execute an instruction and return a new state
///
/// `RestProg` is the remaining instruction sequence with the current instruction removed.
pub trait Execute<Stack, Locals, Memory, CallStack, RestProg> {
    type OutputState;
}

// --- Helper trait for Stack-only instructions (Adapter Pattern) ---
pub trait RunStep<Stack> {
    type OutputStack: Cons;
}

// Note: Removed generic implementation of Execute for RunStep to avoid conflict with OpCall/OpReturn/OpIf.
// Instead, we will implement Execute for each Stack-only instruction using a macro or explicitly.

macro_rules! impl_execute_via_runstep {
    ($Inst:ty) => {
        impl<Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for $Inst
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
        {
            type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Locals, Memory, CallStack, RestProg>;
        }
    };
    ($Inst:ident < $($T:ident),+ >) => {
        impl<$($T),+, Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for $Inst<$($T),+>
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
        {
            type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Locals, Memory, CallStack, RestProg>;
        }
    };
}

impl<N, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpPush<N>
where
    Self: RunStep<Stack>,
    RestProg: Cons,
{
    type OutputState =
        MachineState<<Self as RunStep<Stack>>::OutputStack, Locals, Memory, CallStack, RestProg>;
}
impl_execute_via_runstep!(OpAdd);
impl_execute_via_runstep!(OpSub);
impl_execute_via_runstep!(OpDup);
impl_execute_via_runstep!(OpSwap);
impl_execute_via_runstep!(OpDrop);
impl_execute_via_runstep!(OpEq);
impl_execute_via_runstep!(OpNeq);
impl_execute_via_runstep!(OpLt);
impl_execute_via_runstep!(OpGt);
impl_execute_via_runstep!(OpNot);
impl_execute_via_runstep!(OpAnd);
impl_execute_via_runstep!(OpOr);

// Note: OpLet is not implemented via RunStep because it modifies Locals.

//
// Instruction Implementations
//

// --- OpPush<N> ---
impl<Val, Stack> RunStep<Stack> for OpPush<Val>
where
    Stack: Cons,
{
    type OutputStack = TyArray<Val, Stack>;
}

// --- OpAdd ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpAdd
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::int::EAdd<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::int::EAdd<Lhs, Rhs>>, RestStack>;
}

// --- OpSub ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpSub
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::int::ESub<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::int::ESub<Lhs, Rhs>>, RestStack>;
}

// --- OpDup ---
impl<Val, RestStack> RunStep<TyArray<Val, RestStack>> for OpDup
where
    TyArray<Val, RestStack>: Cons,
    RestStack: Cons,
{
    type OutputStack = TyArray<Val, TyArray<Val, RestStack>>;
}

// --- OpSwap ---
impl<Top, Second, RestStack> RunStep<TyArray<Top, TyArray<Second, RestStack>>> for OpSwap
where
    TyArray<Top, TyArray<Second, RestStack>>: Cons,
    RestStack: Cons,
{
    type OutputStack = TyArray<Second, TyArray<Top, RestStack>>;
}

// --- OpDrop ---
impl<Val, RestStack> RunStep<TyArray<Val, RestStack>> for OpDrop
where
    TyArray<Val, RestStack>: Cons,
    RestStack: Cons,
{
    type OutputStack = RestStack;
}

// --- OpEq ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpEq
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::cmp::EEq<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::cmp::EEq<Lhs, Rhs>>, RestStack>;
}

// --- OpNeq ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpNeq
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::cmp::ENeq<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::cmp::ENeq<Lhs, Rhs>>, RestStack>;
}

// --- OpLt ---
// Stack: [Rhs, Lhs, ...] -> Push Lhs < Rhs
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpLt
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::cmp::ELt<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::cmp::ELt<Lhs, Rhs>>, RestStack>;
}

// --- OpGt ---
// Stack: [Rhs, Lhs, ...] -> Push Lhs > Rhs
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpGt
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::cmp::EGt<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::cmp::EGt<Lhs, Rhs>>, RestStack>;
}

// --- OpNot ---
impl<Val, RestStack> RunStep<TyArray<Val, RestStack>> for OpNot
where
    TyArray<Val, RestStack>: Cons,
    RestStack: Cons,
    crate::std::bool::ENot<Val>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::bool::ENot<Val>>, RestStack>;
}

// --- OpAnd ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpAnd
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::bool::EAnd<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::bool::EAnd<Lhs, Rhs>>, RestStack>;
}

// --- OpOr ---
impl<Lhs, Rhs, RestStack> RunStep<TyArray<Rhs, TyArray<Lhs, RestStack>>> for OpOr
where
    TyArray<Rhs, TyArray<Lhs, RestStack>>: Cons,
    RestStack: Cons,
    crate::std::bool::EOr<Lhs, Rhs>: Eval,
{
    type OutputStack = TyArray<Evaluate<crate::std::bool::EOr<Lhs, Rhs>>, RestStack>;
}

// --- OpLoad ---
// Stack: [Addr, ...] -> Memory, ... -> Stack: [Value, ...]
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Addr, RestStack>, Locals, Memory, CallStack, RestProg> for OpLoad
where
    Addr: Unsigned,
    TyArray<Addr, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Get<Addr>,
{
    type OutputState = MachineState<
        TyArray<<Memory as Get<Addr>>::Output, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

// --- OpStore ---
// Stack: [Value, Addr, ...] -> Memory, ... -> Memory[Addr] = Value, Stack: [...]
impl<Value, Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Value, TyArray<Addr, RestStack>>, Locals, Memory, CallStack, RestProg>
    for OpStore
where
    Addr: Unsigned,
    TyArray<Value, TyArray<Addr, RestStack>>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Set<Addr, Value>,
{
    type OutputState =
        MachineState<RestStack, Locals, <Memory as Set<Addr, Value>>::Output, CallStack, RestProg>;
}

// --- OpGetLocal<Idx> ---
impl<Idx, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpGetLocal<Idx>
where
    Idx: Unsigned,
    Stack: Cons,
    Locals: Get<Idx>,
    RestProg: Cons,
{
    type OutputState = MachineState<
        TyArray<<Locals as Get<Idx>>::Output, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

// --- OpSetLocal<Idx> ---
impl<Idx, Value, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpSetLocal<Idx>
where
    Idx: Unsigned,
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons, // Requirement for TyArray<Value, RestStack>
    Locals: Set<Idx, Value>,
    RestProg: Cons,
{
    type OutputState =
        MachineState<RestStack, <Locals as Set<Idx, Value>>::Output, Memory, CallStack, RestProg>;
}

// --- OpLet ---
impl<Value, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpLet
where
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons,
    Locals: Cons,
    RestProg: Cons,
{
    // Locals -> TyArray<Value, Locals> (Prepend)
    type OutputState =
        MachineState<RestStack, TyArray<Value, Locals>, Memory, CallStack, RestProg>;
}

// --- OpDropLocal ---
impl<Stack, Head, Tail, Memory, CallStack, RestProg>
    Execute<Stack, TyArray<Head, Tail>, Memory, CallStack, RestProg> for OpDropLocal
where
    Stack: Cons,
    TyArray<Head, Tail>: Cons,
    Tail: Cons,
    RestProg: Cons,
{
    // Locals -> Tail (Remove Head)
    type OutputState = MachineState<Stack, Tail, Memory, CallStack, RestProg>;
}

// --- OpCall<Prog> ---
// CallStack: [...] -> [(RestProg, CallerLocals), ...]
// Program: Prog
// Locals: Reset to empty (or we could keep them, but isolation is safer).
// For now, let's reset to TyNil.
impl<Prog, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpCall<Prog>
where
    Stack: Cons,
    Locals: Cons, // Locals must be Cons (TyNil is Cons) to be put in TyArray
    Memory: Cons,
    CallStack: Cons,
    Prog: Cons,
    RestProg: Cons,
{
    // CallStack: TyArray<TyArray<RestProg, Locals>, RestCallStack>
    // Frame = TyArray<RestProg, Locals>
    // Note: Locals is a list (Cons). RestProg is a list (Cons).
    // TyArray<RestProg, Locals> means Head=RestProg, Tail=Locals.
    // This assumes Locals is a VALID Tail, which implies Locals: Cons.
    type OutputState =
        MachineState<Stack, TyNil, Memory, TyArray<TyArray<RestProg, Locals>, CallStack>, Prog>;
}

// --- OpReturn ---
// CallStack: [[Continuation, CallerLocals], RestCallStack...] -> RestCallStack
// Program: Continuation
// Locals: CallerLocals
impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg>
    Execute<
        Stack,
        Locals,
        Memory,
        TyArray<TyArray<Continuation, CallerLocals>, RestCallStack>,
        RestProg,
    > for OpReturn
where
    Stack: Cons,
    Memory: Cons,
    TyArray<TyArray<Continuation, CallerLocals>, RestCallStack>: Cons,
    TyArray<Continuation, CallerLocals>: Cons,
    Continuation: Cons,
    CallerLocals: Cons, // CallerLocals must be Cons to be used as Locals
    RestCallStack: Cons,
    RestProg: Cons,
{
    type OutputState = MachineState<Stack, CallerLocals, Memory, RestCallStack, Continuation>;
}

// --- OpWhile<Cond, Body> ---
// Expansion: Cond + [OpIf<Body + [OpWhile<Cond, Body>], []>] + RestProg
impl<Cond, Body, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpWhile<Cond, Body>
where
    Stack: Cons,
    Locals: Cons,
    Cond: Cons,
    Body: Cons,
    RestProg: Cons,
    EConcat<Body, TyArray<OpWhile<Cond, Body>, TyNil>>: Eval,
    // Alias for the recursive body: Body + [While]
    EConcat<Body, TyArray<OpWhile<Cond, Body>, TyNil>>: Eval,
    // Note: Use 'Evaluator<...>' for the recursive part to ensure it is treated as a type, not an expression that confuses the parser
    EConcat<
        Cond,
        TyArray<
            OpIf<Evaluate<EConcat<Body, TyArray<OpWhile<Cond, Body>, TyNil>>>, TyNil>,
            RestProg,
        >,
    >: Eval,
{
    type OutputState = MachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        Evaluate<
            EConcat<
                Cond,
                TyArray<
                    OpIf<Evaluate<EConcat<Body, TyArray<OpWhile<Cond, Body>, TyNil>>>, TyNil>,
                    RestProg,
                >,
            >,
        >,
    >;
}

// --- OpIf<Then, Else> ---
impl<Cond, RestStack, Locals, Memory, CallStack, Then, Else, RestProg>
    Execute<TyArray<Cond, RestStack>, Locals, Memory, CallStack, RestProg> for OpIf<Then, Else>
where
    TyArray<Cond, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Then: Cons,
    Else: Cons,
    EConcat<Then, RestProg>: Eval,
    EConcat<Else, RestProg>: Eval,
    EIf<
        Cond,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Then, RestProg>>>,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Else, RestProg>>>,
    >: Eval,
{
    type OutputState = Evaluate<
        EIf<
            Cond,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Then, RestProg>>>,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Else, RestProg>>>,
        >,
    >;
}

//
// Machine Execution (Step / Run)
//

/// Execute one step
/// MachineState<Stack, Locals, Memory, CallStack, Cons<Inst, RestProg>> -> NewState
pub struct OpStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg>
    Apply<MachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>>> for OpStep
where
    Inst: Execute<Stack, Locals, Memory, CallStack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    // Wrap result in ELit because Apply returns an Expression
    type Output = ELit<<Inst as Execute<Stack, Locals, Memory, CallStack, RestProg>>::OutputState>;
}

// OpStep for ELit-wrapped state (handles subsequent loop iterations)
impl<Stack, Locals, Memory, CallStack, Inst, RestProg>
    Apply<ELit<MachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>>>> for OpStep
where
    Inst: Execute<Stack, Locals, Memory, CallStack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = ELit<<Inst as Execute<Stack, Locals, Memory, CallStack, RestProg>>::OutputState>;
}

// --- OpIsFinished: Check if program is empty ---
pub struct OpIsFinished;

impl<S, L, M, C, P> Apply<MachineState<S, L, M, C, P>> for OpIsFinished
where
    crate::std::array::EIsEmpty<ELit<P>>: Eval,
    // We need to evaluate IsEmpty(P) and then Not it.
    // EApp<OpNot, EApp<OpIsEmpty, P>>
    EApp<crate::std::bool::OpNot, crate::std::array::EIsEmpty<ELit<P>>>: Eval,
{
    type Output = EApp<crate::std::bool::OpNot, crate::std::array::EIsEmpty<ELit<P>>>;
}

// OpIsFinished for ELit-wrapped state (handles subsequent loop iterations)
impl<S, L, M, C, P> Apply<ELit<MachineState<S, L, M, C, P>>> for OpIsFinished
where
    crate::std::array::EIsEmpty<ELit<P>>: Eval,
    EApp<crate::std::bool::OpNot, crate::std::array::EIsEmpty<ELit<P>>>: Eval,
{
    type Output = EApp<crate::std::bool::OpNot, crate::std::array::EIsEmpty<ELit<P>>>;
}

// --- Machine Runner ---
// Run<InitialState> -> FinalState
// uses EWhile<Condition, Step, State>

pub type ERun<S> = EWhile<OpIsFinished, OpStep, S>;

//
// Tests
//

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3, U5};

    use super::*;
    use crate::tyarray;

    #[test]
    fn test_stack_ops_logic() {
        type S0 = TyNil;
        // Push 1
        type S1 = <OpPush<U1> as RunStep<S0>>::OutputStack;
        assert_type_eq_all!(S1, tyarray![U1]);
    }

    #[test]
    fn test_machine_run() {
        // Program: Push 2, Push 3, Add, Push 5, Sub
        // [2] -> [3, 2] -> [5] -> [5, 5] -> [0]

        type Prog = tyarray![OpPush<U2>, OpPush<U3>, OpAdd, OpPush<U5>, OpSub];

        // Empty Memory, Empty CallStack, Empty Locals
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_call_return() {
        // Subroutine: Push 5, Return
        type SubRoutine = tyarray![OpPush<U5>, OpReturn];

        // Main: Push 3, Call SubRoutine, Add
        // [3] -> Call -> [5, 3] -> [8]
        type MainProg = tyarray![OpPush<U3>, OpCall<SubRoutine>, OpAdd];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, MainProg>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U8];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_memory_and_function() {
        use typenum::{U10, U20};

        // Initialize memory with 0 at address 0
        type InitialMemory = tyarray![typenum::U0];

        // Function: Increment value at address 0
        // Stack: []
        // Logic: Push 0 (addr), Push 10 (val), Store
        //        Push 0 (addr), Load, Push 20 (val), Add
        //        Push 0 (addr), Swap, Store, Return
        type Func = tyarray![
            OpPush<typenum::U0>, // Addr
            OpPush<U10>,         // Val
            OpStore,             // [Val, Addr] -> Mem[Addr]=Val. Stack: []
            OpPush<typenum::U0>, // Addr
            OpLoad,              // [Addr] -> [Val] (10)
            OpPush<U20>,         // [20, 10]
            OpAdd,               // [30]
            OpPush<typenum::U0>, // [0, 30] (Addr, Val)
            OpSwap,              // [30, 0] (Val, Addr)
            OpStore,             // Mem[0]=30. Stack: []
            OpReturn
        ];

        // Main: Call Func
        type Main = tyarray![OpCall<Func>];

        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;

        // Expected Memory: [30] (10 + 20)
        type ExpectedMemory = tyarray![typenum::U30];
        type ExpectedState = MachineState<TyNil, TyNil, ExpectedMemory, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_while_loop_countdown() {
        use typenum::{U0, U1, U3, U10};

        // Loop: While StackTop > 0, Decrement
        // Stack: [N]
        // Cond: Dup, Push 0, Gt (Top > 0)
        // Body: Push 1, Sub

        type CondProg = tyarray![OpDup, OpPush<U0>, OpGt];

        type BodyProg = tyarray![OpPush<U1>, OpSub];

        type Prog = tyarray![
            OpPush<U3>, // Start at 3
            OpWhile<CondProg, BodyProg>
        ];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluate<ERun<InitialState>>;

        // Result should be 0
        type ExpectedStack = tyarray![U0];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_simple_sort() {
        use typenum::{U0, U1, U3};
        // Sort memory [3, 1] -> [1, 3] at indices 0, 1

        type InitialMemory = tyarray![U3, U1];

        // Algorithm:
        // Load 0 (ValA), Load 1 (ValB).
        // If ValA > ValB:
        //   Store ValA at 1, Store ValB at 0.
        // Else:
        //   Keep.

        // Stack: []
        // Push 0, Load -> [3]
        // Push 1, Load -> [1, 3]
        // Dup, Push 3 (index 0, we can't random access easily without re-loading or keeping indices)
        // Let's keep it simple: Load A, Load B.
        // If A > B: Swap in memory.

        // Condition: Push 0, Load, Push 1, Load, Gt
        // Body: Push 0, Load, Push 1, Load, Swap, Push 0, Swap, Store, Push 1, Swap, Store
        // Wait, "Swap in memory" logic:
        // Stack: [B, A]. We want Mem[0]=B, Mem[1]=A.
        // Push 1 (Addr), Swap (-> [1, B, A]). Store (Mem[1]=A). Stack: [B].
        // Push 0 (Addr), Swap (-> [0, B]). Store (Mem[0]=B). Stack: [].

        type SortProg = tyarray![
            // Check if Mem[0] > Mem[1]
            OpPush<U0>, OpLoad,
            OpPush<U1>, OpLoad,
            OpGt,
            // If True: Swap them
            OpIf<
                tyarray![
                    OpPush<U0>, OpLoad, // A
                    OpPush<U1>, OpLoad, // B
                    // Stack: [B, A] (e.g., [1, 3])
                    OpSwap, // [A, B] ([3, 1])
                    OpPush<U1>, // [1, A, B]
                    OpSwap,     // [A, 1, B] (Val=A, Addr=1)
                    OpStore,    // Mem[1]=A. Stack: [B]
                    OpPush<U0>, // [0, B]
                    OpSwap,     // [B, 0] (Val=B, Addr=0)
                    OpStore     // Mem[0]=B. Stack: []
                ],
                tyarray![] // Else: Do nothing
            >
        ];

        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, SortProg>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedMemory = tyarray![U1, U3];
        type ExpectedState = MachineState<TyNil, TyNil, ExpectedMemory, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_locals_isolation() {
        use typenum::{U0, U10, U20};

        // Test Locals Isolation
        // Main -> A -> B
        // A: Let x = 10. Call B. Check x == 10.
        // B: Let x = 20. Return.

        type FuncB = tyarray![
            OpPush<U20>,
            OpLet, // Define Local[0] = 20 (Head of B's locals)
            OpReturn
        ];

        type FuncA = tyarray![
            OpPush<U10>,
            OpLet,         // Define Local[0] = 10 (Head of A's locals)
            OpCall<FuncB>, // Call B
            // B returns. A's locals should be restored.
            // OpGetLocal pushes to stack. We don't need to push index to stack.
            OpGetLocal<U0>, // Should get 10
            OpReturn
        ];

        type Main = tyarray![OpCall<FuncA>];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;

        // FinalStack should contain [10] (result of GetLocal)
        type ExpectedStack = tyarray![U10];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
