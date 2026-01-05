//! Large-scale integration tests for the traced state machine.
//!
//! These tests verify that the `TracedExecute` trait correctly records
//! instruction history during complex multi-step execution.
//!
//! Note: Single instruction tests are unit tests in the implementation modules.
//! Property-based tests and compile-fail tests should use proptest/trybuild.

// Allow dead_code for intermediate type aliases in multi-step test chains
#![allow(dead_code)]

use static_assertions::assert_type_eq_all;
use typelude_core::{ELit, Evaluate};
use typelude_std::{
    std::{
        col::array::Nil,
        // Import operators from typelude_std::std::ops for TracedExecute
        ops::OpAdd as StdOpAdd,
        trace::Trace,
    },
    tyarray,
};
use typelude_vm::machine::{
    instruction::*,
    trace::{ETracedRun, TracedExecute, TracedMachineState},
};
use typenum::{U0, U1, U2, U3};

// ============================================================================
// Large Integration Test: Manual Multi-Step Execution with History
// ============================================================================

/// Integration test: Manually chain multiple TracedExecute steps and verify
/// history accumulates correctly.
///
/// Program simulation:
/// 1. Push(1)
/// 2. Push(2)
/// 3. Add
///
/// Expected final state:
/// - Stack: [3]
/// - History: [Add, Push(2), Push(1)]
#[test]
fn test_traced_multi_step_execution() {
    // Step 1: Push(1) on empty state
    type S0 = Nil;
    type H0 = Nil;
    type S1 = <OpPush<ELit<U1>> as TracedExecute<S0, Nil, Nil, Nil, Nil, H0>>::OutputState;

    // Verify intermediate state
    type ExpectedStack1 = tyarray![ELit<U1>];
    type ExpectedHistory1 = tyarray![OpPush<ELit<U1>>];
    type Expected1 = TracedMachineState<ExpectedStack1, Nil, Nil, Nil, Nil, ExpectedHistory1>;
    assert_type_eq_all!(S1, Expected1);

    // Step 2: Push(2) on S1
    type S2 = <OpPush<ELit<U2>> as TracedExecute<
        ExpectedStack1,
        Nil,
        Nil,
        Nil,
        Nil,
        ExpectedHistory1,
    >>::OutputState;

    // Verify intermediate state
    type ExpectedStack2 = tyarray![ELit<U2>, ELit<U1>];
    type ExpectedHistory2 = tyarray![OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type Expected2 = TracedMachineState<ExpectedStack2, Nil, Nil, Nil, Nil, ExpectedHistory2>;
    assert_type_eq_all!(S2, Expected2);

    // Step 3: Add on S2
    type S3 = <StdOpAdd as TracedExecute<ExpectedStack2, Nil, Nil, Nil, Nil, ExpectedHistory2>>::OutputState;

    // Verify final state
    type ExpectedStack3 = tyarray![U3]; // 2 + 1 = 3
    type ExpectedHistory3 = tyarray![StdOpAdd, OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type Expected3 = TracedMachineState<ExpectedStack3, Nil, Nil, Nil, Nil, ExpectedHistory3>;
    assert_type_eq_all!(S3, Expected3);

    // Verify trace output
    let trace = <S3 as Trace>::fmt();
    assert!(trace.contains("[3]"), "Stack should show [3], got: {}", trace);
    assert!(
        trace.contains("Add") && trace.contains("Push(2)") && trace.contains("Push(1)"),
        "History should show [Add, Push(2), Push(1)], got: {}",
        trace
    );
}

/// Integration test: Stack manipulation with Dup, Swap, Drop
///
/// Program simulation:
/// 1. Push(1)
/// 2. Push(2)
/// 3. Dup     -> [2, 2, 1]
/// 4. Swap    -> [2, 2, 1] (swaps top two: [2, 2, 1])
/// 5. Drop    -> [2, 1]
///
/// Expected final state:
/// - Stack: [2, 1]
/// - History: [Drop, Swap, Dup, Push(2), Push(1)]
#[test]
fn test_traced_stack_manipulation_sequence() {
    // Step 1: Push(1)
    type S1 = <OpPush<ELit<U1>> as TracedExecute<Nil, Nil, Nil, Nil, Nil, Nil>>::OutputState;

    // Step 2: Push(2)
    type Stack1 = tyarray![ELit<U1>];
    type History1 = tyarray![OpPush<ELit<U1>>];
    type S2 =
        <OpPush<ELit<U2>> as TracedExecute<Stack1, Nil, Nil, Nil, Nil, History1>>::OutputState;

    // Step 3: Dup
    type Stack2 = tyarray![ELit<U2>, ELit<U1>];
    type History2 = tyarray![OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type S3 = <OpDup as TracedExecute<Stack2, Nil, Nil, Nil, Nil, History2>>::OutputState;

    // Verify after Dup: Stack = [2, 2, 1]
    type ExpectedStack3 = tyarray![ELit<U2>, ELit<U2>, ELit<U1>];
    type History3 = tyarray![OpDup, OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type Expected3 = TracedMachineState<ExpectedStack3, Nil, Nil, Nil, Nil, History3>;
    assert_type_eq_all!(S3, Expected3);

    // Step 4: Swap (swaps top two: [2, 2, 1] -> [2, 2, 1])
    type S4 = <OpSwap as TracedExecute<ExpectedStack3, Nil, Nil, Nil, Nil, History3>>::OutputState;

    // After Swap (swapping identical values): [2, 2, 1]
    type Stack4 = tyarray![ELit<U2>, ELit<U2>, ELit<U1>];
    type History4 = tyarray![OpSwap, OpDup, OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type Expected4 = TracedMachineState<Stack4, Nil, Nil, Nil, Nil, History4>;
    assert_type_eq_all!(S4, Expected4);

    // Step 5: Drop
    type S5 = <OpDrop as TracedExecute<Stack4, Nil, Nil, Nil, Nil, History4>>::OutputState;

    // Verify final state: Stack = [2, 1], History = [Drop, Swap, Dup, Push(2),
    // Push(1)]
    type ExpectedStack5 = tyarray![ELit<U2>, ELit<U1>];
    type ExpectedHistory5 = tyarray![OpDrop, OpSwap, OpDup, OpPush<ELit<U2>>, OpPush<ELit<U1>>];
    type Expected5 = TracedMachineState<ExpectedStack5, Nil, Nil, Nil, Nil, ExpectedHistory5>;
    assert_type_eq_all!(S5, Expected5);

    // Verify trace output
    let trace = <S5 as Trace>::fmt();
    assert!(
        trace.contains("Drop") && trace.contains("Swap") && trace.contains("Dup"),
        "History should contain stack manipulation ops, got: {}",
        trace
    );
}

/// Integration test: Local variable operations
///
/// Program simulation:
/// 1. Push(3)
/// 2. Let      -> Locals = [3]
/// 3. GetLocal(0) -> Stack = [3]
///
/// Expected final state:
/// - Stack: [3]
/// - Locals: [3]
/// - History: [GetLocal(0), Let, Push(3)]
#[test]
fn test_traced_local_variable_operations() {
    // Step 1: Push(3)
    type S1 = <OpPush<ELit<U3>> as TracedExecute<Nil, Nil, Nil, Nil, Nil, Nil>>::OutputState;

    // Step 2: Let
    type Stack1 = tyarray![ELit<U3>];
    type History1 = tyarray![OpPush<ELit<U3>>];
    type S2 = <OpLet as TracedExecute<Stack1, Nil, Nil, Nil, Nil, History1>>::OutputState;

    // Verify: Stack = [], Locals = [3]
    type Locals2 = tyarray![ELit<U3>];
    type History2 = tyarray![OpLet, OpPush<ELit<U3>>];
    type Expected2 = TracedMachineState<Nil, Locals2, Nil, Nil, Nil, History2>;
    assert_type_eq_all!(S2, Expected2);

    // Step 3: GetLocal(0)
    type S3 =
        <OpGetLocal<U0> as TracedExecute<Nil, Locals2, Nil, Nil, Nil, History2>>::OutputState;

    // Verify final state
    type ExpectedStack3 = tyarray![ELit<U3>];
    type ExpectedHistory3 = tyarray![OpGetLocal<U0>, OpLet, OpPush<ELit<U3>>];
    type Expected3 = TracedMachineState<ExpectedStack3, Locals2, Nil, Nil, Nil, ExpectedHistory3>;
    assert_type_eq_all!(S3, Expected3);

    // Verify trace output
    let trace = <S3 as Trace>::fmt();
    assert!(trace.contains("Locals: [3]"), "Locals should show [3], got: {}", trace);
    assert!(trace.contains("GetLocal"), "History should contain GetLocal, got: {}", trace);
}

/// Integration test: Call and Return with history preservation
///
/// Simulates:
/// 1. Push(5)
/// 2. Call<SubRoutine> where SubRoutine = [Push(10), Return]
/// 3. Return restores context
///
/// Note: This tests the Call instruction's effect on history,
/// not a full run which would require ETracedRun.
#[test]
fn test_traced_call_instruction() {
    type SubRoutine = tyarray![OpPush<ELit<U3>>, OpReturn];

    // Step 1: Push(5)
    type S1 = <OpPush<ELit<U1>> as TracedExecute<Nil, Nil, Nil, Nil, Nil, Nil>>::OutputState;

    // Step 2: Call<SubRoutine>
    type Stack1 = tyarray![ELit<U1>];
    type Locals1 = Nil;
    type RestProg1 = tyarray![StdOpAdd]; // The continuation after call
    type History1 = tyarray![OpPush<ELit<U1>>];

    type S2 = <OpCall<SubRoutine> as TracedExecute<
        Stack1,
        Locals1,
        Nil,
        Nil,
        RestProg1,
        History1,
    >>::OutputState;

    // Verify Call's effect:
    // - Stack unchanged
    // - Locals reset to Nil
    // - CallStack = [(RestProg, CallerLocals)]
    // - Program = SubRoutine
    // - History = [Call<SubRoutine>, Push(1)]
    type ExpectedCallStack = tyarray![(RestProg1, Locals1)];
    type ExpectedHistory2 = tyarray![OpCall<SubRoutine>, OpPush<ELit<U1>>];
    type Expected2 =
        TracedMachineState<Stack1, Nil, Nil, ExpectedCallStack, SubRoutine, ExpectedHistory2>;

    assert_type_eq_all!(S2, Expected2);

    // Verify trace output shows Call in history
    let trace = <S2 as Trace>::fmt();
    assert!(trace.contains("Call"), "History should contain Call, got: {}", trace);
}

// ============================================================================
// Complex Manual Chain Scenario
// ============================================================================
/// This test manually steps through the Calculator Program to verify
/// correctness of state transitions without relying on EWhile executor.
///
/// Logic:
/// 1. Push(10), Let -> Locals[10]
/// 2. Push(U0), Push(20), Store -> Mem[0]=20
/// 3. Push(5), Call(Sub) Sub: Push(U0), Load, Add, Return -> Result 25 (20+5)
/// 4. GetLocal(0) -> 10
/// 5. Add -> 35
#[test]
fn test_complex_manual_chain() {
    use typenum::{U0, U5, U10, U20, U35};
    type Val10 = ELit<U10>;
    type Val20 = ELit<U20>;
    type Val5 = ELit<U5>;

    // Explicit SubRoutine definition
    type SubRoutine = tyarray![OpPush<U0>, OpLoad, StdOpAdd, OpReturn];

    // --- Steps ---
    // Start State
    type InitMem = tyarray![ELit<U0>];
    type H0 = Nil;

    // 1. Push(10)
    type S1 = <OpPush<Val10> as TracedExecute<Nil, Nil, InitMem, Nil, Nil, H0>>::OutputState;
    // S1: Stack=[10]

    // 2. Let
    use typelude_vm::machine::state::GetStack;
    type Stack1 = tyarray![Val10];
    type History1 = tyarray![OpPush<Val10>];
    type S2 = <OpLet as TracedExecute<Stack1, Nil, InitMem, Nil, Nil, History1>>::OutputState;
    // S2: Stack=[], Locals=[10]

    // 3. Push(U0)
    type Locals2 = tyarray![Val10];
    type History2 = tyarray![OpLet, OpPush<Val10>];
    type S3 =
        <OpPush<U0> as TracedExecute<Nil, Locals2, InitMem, Nil, Nil, History2>>::OutputState;
    // S3: Stack=[U0]

    // 4. Push(20)
    type Stack3 = tyarray![U0];
    type History3 = tyarray![OpPush<U0>, OpLet, OpPush<Val10>];
    type S4 = <OpPush<Val20> as TracedExecute<Stack3, Locals2, InitMem, Nil, Nil, History3>>::OutputState;
    // S4: Stack=[20, U0]

    // 5. Store
    type Stack4 = tyarray![Val20, U0];
    type History4 = tyarray![OpPush<Val20>, OpPush<U0>, OpLet, OpPush<Val10>];
    type S5 =
        <OpStore as TracedExecute<Stack4, Locals2, InitMem, Nil, Nil, History4>>::OutputState;
    // S5: Stack=[], Mem=[20]
    type Mem5 = tyarray![Val20];

    // 6. Push(5) (Arg)
    type History5 = tyarray![OpStore, OpPush<Val20>, OpPush<U0>, OpLet, OpPush<Val10>];
    type S6 = <OpPush<Val5> as TracedExecute<Nil, Locals2, Mem5, Nil, Nil, History5>>::OutputState;
    // S6: Stack=[5]

    // 7. Call(Sub)
    // IMPORTANT: The generic `TargetProg` for Call must match.
    // RestProg here is 'continuation'. In manual mode, we define continuation
    // manually. Let's assume Call pushes (RestProg, Locals) to CallStack.
    // We define RestProg types later?
    // Actually, in manual step types, `RestProg` is just a placeholder `Nil` in our
    // steps 1-6 above because we didn't specify RestProg. But OpCall PUSHES
    // that placeholder to CallStack. If we want Return to jump back to
    // something relevant, we have to fake the Continuation. Let's use a dummy
    // continuation `Nil`. We will execute the 'after return' steps manually too.
    type Stack6 = tyarray![Val5];
    type History6 =
        tyarray![OpPush<Val5>, OpStore, OpPush<Val20>, OpPush<U0>, OpLet, OpPush<Val10>];
    type S7 = <OpCall<SubRoutine> as TracedExecute<Stack6, Locals2, Mem5, Nil, Nil, History6>>::OutputState;
    // S7: Prog=SubRoutine. CallStack=[(Nil, Locals2)]. Locals=Nil.
    type CallStack7 = tyarray![(Nil, Locals2)];
    type History7 = tyarray![
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];

    // --- SubRoutine Execution ---
    // Sub: [Push(U0), Load, Add, Return]

    // 8. Push(U0)
    // Context: Locals=Nil, Mem=Mem5, CallStack=CallStack7.
    type S8 =
        <OpPush<U0> as TracedExecute<Stack6, Nil, Mem5, CallStack7, Nil, History7>>::OutputState;
    // Stack=[U0, 5]

    // 9. Load
    type Stack8 = tyarray![U0, Val5];
    type History8 = tyarray![
        OpPush<U0>,
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];
    type S9 = <OpLoad as TracedExecute<Stack8, Nil, Mem5, CallStack7, Nil, History8>>::OutputState;
    // Stack=[20, 5]

    // 10. Add
    type Stack9 = tyarray![Val20, Val5];
    type History9 = tyarray![
        OpLoad,
        OpPush<U0>,
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];
    type S10 =
        <StdOpAdd as TracedExecute<Stack9, Nil, Mem5, CallStack7, Nil, History9>>::OutputState;
    // Stack=[25]

    // 11. Return
    type Stack10 = tyarray![ELit<typenum::Sum<U20, U5>>];
    type History10 = tyarray![
        StdOpAdd,
        OpLoad,
        OpPush<U0>,
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];
    // Return restores Continuation (Nil) and Locals (Locals2).
    // CallStack: [(Nil, Locals2), Rest...] -> Head is (Nil, Locals2).
    type S11 =
        <OpReturn as TracedExecute<Stack10, Nil, Mem5, CallStack7, Nil, History10>>::OutputState;
    // S11: Locals=Locals2. CallStack=Nil. Stack=[25].

    // --- Post Return ---
    // 12. GetLocal(0)
    // Stack=[25]. Locals=[10].
    type Stack11 = Stack10;
    type History11 = tyarray![
        OpReturn,
        StdOpAdd,
        OpLoad,
        OpPush<U0>,
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];
    type S12 = <OpGetLocal<U0> as TracedExecute<Stack11, Locals2, Mem5, Nil, Nil, History11>>::OutputState;
    // Stack=[10, 25].

    // 13. Add
    type Stack12 = tyarray![Val10, ELit<typenum::Sum<U20, U5>>];
    type History12 = tyarray![
        OpGetLocal<U0>,
        OpReturn,
        StdOpAdd,
        OpLoad,
        OpPush<U0>,
        OpCall<SubRoutine>,
        OpPush<Val5>,
        OpStore,
        OpPush<Val20>,
        OpPush<U0>,
        OpLet,
        OpPush<Val10>
    ];
    type S13 =
        <StdOpAdd as TracedExecute<Stack12, Locals2, Mem5, Nil, Nil, History12>>::OutputState;
    // Stack=[35].

    // Verify Final Trace
    let trace = <S13 as Trace>::fmt();
    println!("Final Trace:\n{}", trace);

    assert!(trace.contains("[35]"), "Should result in 35");
    assert!(trace.contains("Push(10)"));
    assert!(trace.contains("Call"));
    assert!(trace.contains("Load"));
    assert!(trace.contains("Store"));
    assert!(trace.contains("Return"));
    assert!(trace.contains("GetLocal"));
}
