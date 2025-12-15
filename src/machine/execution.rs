//! **Type-Level Stack Machine Execution Logic**
//!
//! マシンの命令実行ロジックとメインループを実装します。

use crate::eval::{EApply, EIf, EWhile, Evaluable, Evaluator};
use crate::std::array::{Cons, Get, Set, TyArray, TyNil};
use crate::std::traits::EFunction;
use crate::std::bool::FNot;
use crate::std::array::{EConcat, FIsEmpty};
// Important: Ensure types::int is imported so Evaluable impls for Arithmetic are visible
#[allow(unused_imports)]
use crate::std::int;
use crate::machine::instruction::*;
use crate::machine::state::MachineState;
use typenum::Unsigned;

// =============================================================================
// Execute Trait
// =============================================================================

/// 命令を実行して新しい状態を返すトレイト
///
/// `RestProg` は、現在の命令を取り除いた残りの命令列。
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

impl<N, Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for OpPush<N>
where
    Self: RunStep<Stack>,
    RestProg: Cons,
{
    type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Locals, Memory, CallStack, RestProg>;
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

// =============================================================================
// Instruction Implementations
// =============================================================================

// --- OpPush<N> ---
impl<N, Stack> RunStep<Stack> for OpPush<N>
where
    Stack: Cons,
{
    type OutputStack = TyArray<N, Stack>;
}

// --- OpAdd ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpAdd
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::int::EAdd<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::int::EAdd<A, B>>, Rest>;
}

// --- OpSub ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpSub
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::int::ESub<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::int::ESub<A, B>>, Rest>;
}

// --- OpDup ---
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDup
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<A, TyArray<A, Rest>>;
}

// --- OpSwap ---
impl<A, B, Rest> RunStep<TyArray<A, TyArray<B, Rest>>> for OpSwap
where
    TyArray<A, TyArray<B, Rest>>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<B, TyArray<A, Rest>>;
}

// --- OpDrop ---
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDrop
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = Rest;
}

// --- OpEq ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpEq
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::cmp::EEq<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::cmp::EEq<A, B>>, Rest>;
}

// --- OpNeq ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpNeq
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::cmp::ENotEq<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::cmp::ENotEq<A, B>>, Rest>;
}

// --- OpLt ---
// Stack: [B, A, ...] -> Push A < B
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpLt
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::cmp::ELt<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::cmp::ELt<A, B>>, Rest>;
}

// --- OpGt ---
// Stack: [B, A, ...] -> Push A > B
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpGt
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::cmp::EGt<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::cmp::EGt<A, B>>, Rest>;
}

// --- OpNot ---
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpNot
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
    crate::std::bool::ENot<A>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::bool::ENot<A>>, Rest>;
}

// --- OpAnd ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpAnd
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::bool::EAnd<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::bool::EAnd<A, B>>, Rest>;
}

// --- OpOr ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpOr
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::std::bool::EOr<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::std::bool::EOr<A, B>>, Rest>;
}

// --- OpLoad ---
// Stack: [Addr, ...] -> Memory, ... -> Stack: [Value, ...]
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg> Execute<TyArray<Addr, RestStack>, Locals, Memory, CallStack, RestProg> for OpLoad
where
    Addr: Unsigned,
    TyArray<Addr, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Get<Addr>,
{
    type OutputState = MachineState<TyArray<<Memory as Get<Addr>>::Output, RestStack>, Locals, Memory, CallStack, RestProg>;
}

// --- OpStore ---
// Stack: [Value, Addr, ...] -> Memory, ... -> Memory[Addr] = Value, Stack: [...]
impl<Value, Addr, RestStack, Locals, Memory, CallStack, RestProg> Execute<TyArray<Value, TyArray<Addr, RestStack>>, Locals, Memory, CallStack, RestProg> for OpStore
where
    Addr: Unsigned,
    TyArray<Value, TyArray<Addr, RestStack>>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Set<Addr, Value>,
{
    type OutputState = MachineState<RestStack, Locals, <Memory as Set<Addr, Value>>::Output, CallStack, RestProg>;
}

// --- OpGetLocal<Index> ---
impl<Index, Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for OpGetLocal<Index>
where
    Index: Unsigned,
    Stack: Cons,
    Locals: Get<Index>,
    RestProg: Cons,
{
    type OutputState = MachineState<TyArray<<Locals as Get<Index>>::Output, Stack>, Locals, Memory, CallStack, RestProg>;
}

// --- OpSetLocal<Index> ---
impl<Index, Value, RestStack, Locals, Memory, CallStack, RestProg> Execute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpSetLocal<Index>
where
    Index: Unsigned,
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons, // Requirement for TyArray<Value, RestStack>
    Locals: Set<Index, Value>,
    RestProg: Cons,
{
    type OutputState = MachineState<RestStack, <Locals as Set<Index, Value>>::Output, Memory, CallStack, RestProg>;
}

// --- OpLet ---
impl<Value, RestStack, Locals, Memory, CallStack, RestProg> Execute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpLet
where
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons,
    Locals: Cons,
    RestProg: Cons,
{
    // Locals -> TyArray<Value, Locals> (Prepend)
    type OutputState = MachineState<RestStack, TyArray<Value, Locals>, Memory, CallStack, RestProg>;
}

// --- OpDropLocal ---
impl<Stack, Head, Tail, Memory, CallStack, RestProg> Execute<Stack, TyArray<Head, Tail>, Memory, CallStack, RestProg> for OpDropLocal
where
    Stack: Cons,
    TyArray<Head, Tail>: Cons,
    Tail: Cons,
    RestProg: Cons,
{
    // Locals -> Tail (Remove Head)
    type OutputState = MachineState<Stack, Tail, Memory, CallStack, RestProg>;
}

// --- OpCall<TargetProg> ---
// CallStack: [...] -> [(RestProg, CallerLocals), ...]
// Program: TargetProg
// Locals: Reset to empty (or we could keep them, but isolation is safer).
// For now, let's reset to TyNil.
impl<TargetProg, Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for OpCall<TargetProg>
where
    Stack: Cons,
    Locals: Cons, // Locals must be Cons (TyNil is Cons) to be put in TyArray
    Memory: Cons,
    CallStack: Cons,
    TargetProg: Cons,
    RestProg: Cons,
{
    // CallStack: TyArray<TyArray<RestProg, Locals>, RestCallStack>
    // Frame = TyArray<RestProg, Locals>
    // Note: Locals is a list (Cons). RestProg is a list (Cons).
    // TyArray<RestProg, Locals> means Head=RestProg, Tail=Locals.
    // This assumes Locals is a VALID Tail, which implies Locals: Cons.
    type OutputState = MachineState<Stack, TyNil, Memory, TyArray<TyArray<RestProg, Locals>, CallStack>, TargetProg>;
}

// --- OpReturn ---
// CallStack: [[Continuation, CallerLocals], RestCallStack...] -> RestCallStack
// Program: Continuation
// Locals: CallerLocals
impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg> Execute<Stack, Locals, Memory, TyArray<TyArray<Continuation, CallerLocals>, RestCallStack>, RestProg> for OpReturn
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

// --- OpWhile<CondProg, BodyProg> ---
// Expansion: CondProg + [OpIf<BodyProg + [OpWhile<CondProg, BodyProg>], []>] + RestProg
impl<CondProg, BodyProg, Stack, Locals, Memory, CallStack, RestProg> Execute<Stack, Locals, Memory, CallStack, RestProg> for OpWhile<CondProg, BodyProg>
where
    Stack: Cons,
    Locals: Cons,
    CondProg: Cons,
    BodyProg: Cons,
    RestProg: Cons,
    EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>: Evaluable,
    // Alias for the recursive body: Body + [While]
    EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>: Evaluable,
    // Note: Use 'Evaluator<...>' for the recursive part to ensure it is treated as a type, not an expression that confuses the parser
    EConcat<CondProg, TyArray<OpIf<Evaluator<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>, TyNil>, RestProg>>: Evaluable,
{
    type OutputState = MachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        Evaluator<EConcat<CondProg, TyArray<OpIf<Evaluator<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>, TyNil>, RestProg>>>
    >;
}

// --- OpIf<Then, Else> ---
impl<Cond, RestStack, Locals, Memory, CallStack, Then, Else, RestProg> Execute<TyArray<Cond, RestStack>, Locals, Memory, CallStack, RestProg>
    for OpIf<Then, Else>
where
    TyArray<Cond, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Then: Cons,
    Else: Cons,
    EConcat<Then, RestProg>: Evaluable,
    EConcat<Else, RestProg>: Evaluable,
    EIf<
        Cond,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluator<EConcat<Then, RestProg>>>,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluator<EConcat<Else, RestProg>>>,
    >: Evaluable,
{
    type OutputState = Evaluator<
        EIf<
            Cond,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluator<EConcat<Then, RestProg>>>,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluator<EConcat<Else, RestProg>>>,
        >,
    >;
}

// =============================================================================
// Machine Execution (Step / Run)
// =============================================================================

/// 1ステップ実行
/// MachineState<Stack, Locals, Memory, CallStack, Cons<Inst, RestProg>> -> NewState
pub struct FStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg> EFunction<MachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>>> for FStep
where
    Inst: Execute<Stack, Locals, Memory, CallStack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = <Inst as Execute<Stack, Locals, Memory, CallStack, RestProg>>::OutputState;
}

// Evaluable wrapper for FStep
impl<S, L, M, C, P> Evaluable for EApply<FStep, MachineState<S, L, M, C, P>>
where
    FStep: EFunction<MachineState<S, L, M, C, P>>,
{
    type Output = <FStep as EFunction<MachineState<S, L, M, C, P>>>::Output;
}

// --- IsFinished: プログラムが空か判定 ---
pub struct FIsFinished;

impl<S, L, M, C, P> EFunction<MachineState<S, L, M, C, P>> for FIsFinished
where
    EApply<FIsEmpty, P>: Evaluable,
    Evaluator<EApply<FIsEmpty, P>>: crate::std::bool::_NotHelper,
{
    type Output = Evaluator<EApply<FNot, EApply<FIsEmpty, P>>>;
}

// --- Machine Runner ---
// Run<InitialState> -> FinalState
// uses EWhile<Condition, Step, State>

pub type ERun<S> = EWhile<FIsFinished, FStep, S>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tyarray;
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3, U5};

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

        type Prog = tyarray![
            OpPush<U2>,
            OpPush<U3>,
            OpAdd,
            OpPush<U5>,
            OpSub
        ];

        // Empty Memory, Empty CallStack, Empty Locals
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_call_return() {
        // Subroutine: Push 5, Return
        type SubRoutine = tyarray![
            OpPush<U5>,
            OpReturn
        ];

        // Main: Push 3, Call SubRoutine, Add
        // [3] -> Call -> [5, 3] -> [8]
        type MainProg = tyarray![
            OpPush<U3>,
            OpCall<SubRoutine>,
            OpAdd
        ];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, MainProg>;
        type FinalState = Evaluator<ERun<InitialState>>;

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
        type Main = tyarray![
            OpCall<Func>
        ];

        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, Main>;
        type FinalState = Evaluator<ERun<InitialState>>;

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

        type CondProg = tyarray![
            OpDup,
            OpPush<U0>,
            OpGt
        ];

        type BodyProg = tyarray![
            OpPush<U1>,
            OpSub
        ];

        type Prog = tyarray![
            OpPush<U3>, // Start at 3
            OpWhile<CondProg, BodyProg>
        ];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;

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
        type FinalState = Evaluator<ERun<InitialState>>;

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
            OpLet,          // Define Local[0] = 10 (Head of A's locals)
            OpCall<FuncB>,  // Call B
            // B returns. A's locals should be restored.
            // OpGetLocal pushes to stack. We don't need to push index to stack.
            OpGetLocal<U0>, // Should get 10
            OpReturn
        ];

        type Main = tyarray![
            OpCall<FuncA>
        ];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Main>;
        type FinalState = Evaluator<ERun<InitialState>>;

        // FinalStack should contain [10] (result of GetLocal)
        type ExpectedStack = tyarray![U10];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
