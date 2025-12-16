# Architecture & Design

Typelude is effectively a **Virtual Machine** specifically architected to run inside the Rust compiler's trait solver.

This document outlines the core components of the machinery.

## 1. System Overview

The system is composed of three distinct layers:

1. **The Hardware (State layer)**: Defines the memory layout (Stack, Registers/Locals, Heap).
2. **The CPU (Evaluator layer)**: The recursive trait resolution engine that executes instructions.
3. **The Assembly (Instruction layer)**: The individual operations (`Add`, `Push`, `Jump`).

## 2. Low-Level Architecture (The "Hardware")

Unlike a physical CPU, our "state" is immutable. Every clock cycle produces a *new type* representing the next state.

### 2.1 The Machine State

Defined in `src/machine/state.rs`, the core state carrier is:

```rust
pub struct MachineState<Stack, Locals, Memory, CallStack, Program> {
    _stack: PhantomData<Stack>,
    _locals: PhantomData<Locals>,
    _memory: PhantomData<Memory>,
    _call_stack: PhantomData<CallStack>,
    _program: PhantomData<Program>,
}
```

* **Stack**: An `HList` (Heterogeneous List) acting as the working memory.
  * `OpPush<N>` adds to the head.
  * `OpAdd` pops two from the head and pushes the result.
* **Locals**: Only stores temporary variables for the *current* frame.
* **Memory**: A global heterogeneous list representing the heap, persistent across function calls.
* **CallStack**: Stores return pointers (Continuations) and the caller's distinct `Locals` frame.
* **Program**: The current instruction tape (a `Cons` list of ops).

---

## 3. The Execution Engine (The "CPU")

The heart of Typelude is the `Execute` trait. This trait implements the **Small-Step Operational Semantics** of the machine.

### 3.1 The Execute Trait

```rust
pub trait Execute<Stack, Locals, Memory, CallStack, RestProg> {
    type OutputState;
}
```

Each instruction implements this trait. The `RestProg` parameter is the *rest of the program* after the current instruction. This creates a chain reaction:

1. The machine looks at `Program = Cons<HeadOp, RestProg>`.
2. It invokes `HeadOp::Execute<..., RestProg>`.
3. The `OutputState` of this execution becomes the input for the next step.

### 3.2 The Adapter Pattern (`RunStep`)

Implementing `Execute` implies handling the entire machine state (Locals, Memory, CallStack, Program flow). For simple math operations like `Add` or `Push`, this is too much boilerplate.

We use an adapter trait `RunStep` for "Stack-Only" operations:

```rust
pub trait RunStep<Stack> {
    type OutputStack: Cons;
}

// Blanket Implementation:
// Any instruction implementing RunStep automatically gets Execute!
impl<Inst, ...> Execute<Stack, ...> for Inst
where Inst: RunStep<Stack>
{
    type OutputState = MachineState<Inst::OutputStack, ...>;
}
```

This allows us to verify simple logic easily.

#### Example: Implementing OpAdd

```rust
// 1. Define the Instruction Marker
pub struct OpAdd;

// 2. Implement Logic
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpAdd
where
    // Ensure the stack has at least 2 elements
    TyArray<B, TyArray<A, Rest>>: Cons,
    // Ensure A and B can be added (Delegating to Typenum or Std lib)
    crate::std::int::EAdd<A, B>: Evaluable,
{
    // The result is the Tail (Rest) with the Sum pushed on front
    type OutputStack = TyArray<
        // Call the helper evaluator
        Evaluator<crate::std::int::EAdd<A, B>>,
        Rest
    >;
}
```

### 3.3 Dispatch & Control Flow

Normal execution is linear. However, control flow (`If`, `While`) requires conditional execution.

We achieve this via **Lazy Normalization** and **Auxiliary Traits**:

* `OpIf<Condition, TrueBranch, FalseBranch>`
* The evaluator calculates `Condition`.
* Based on the result (`True` or `False`), it selects *which branch type to expand*.
* Crucially, the unselected branch is **never instantiated**, preventing infinite recursion in dead code paths.

---

## 4. The Frontend (Macros)

Writing recursive generic structs manually (`Cons<OpPush<U1>...>`) is painful. We provide a Macro DSL to make Typelude readable.

### 4.1 The `program!` Macro

Located in `src/macros/mod.rs`.

This macro acts as a **Compiler-Compiler**. It parses a Rust-like syntax and "transpiles" it into the `Cons<...>` AST structure.

**Input:**

```rust
program! {
    let x = 10;
    push x;
    op_add;
}
```

**Output (simplified):**

```rust
Cons<OpPush<U10>,
    Cons<OpLet<"x">,
        Cons<OpGet<"x">,
            Cons<OpAdd, Nil>
        >
    >
>
```

### 4.2 Handling Variables

The macro layer handles the translation of identifiers (`x`) into symbol types. Because `OpLet` and `OpGet` work on the `Locals` HList, variable shadowing and scoping work exactly as they do in normal Rust, purely as a side effect of the list structure (searching from the head finds the most recently pushed variable first).

---

## 5. Standard Library Design Patterns

This section details the design choices behind the "Functional" part of the library (boolean logic, arithmetic, etc.), specifically how we handle trait dispatch and public APIs.

### 5.1 Trait Implementation Strategies

In Rust's type system, conditional branching is effectively achieved by having the trait solver select different `impl` blocks (or different `Output` types) based on trait bounds. We considered three main patterns:

#### Pattern A: Self-Based Dispatch (Adopted)

The left-hand side (`Lhs`) acts as the subject.

```rust
impl<Lhs, Rhs> TyAnd<Rhs> for Lhs { ... }
```

* **Pros**: Natural syntax (`Lhs.and(Rhs)`).

* **Usage**: Used for most core operations.

#### Pattern B: Unit/Context Dispatch

The unit type `()` or a specific Context object acts as the subject.

```rust
where (): TyAnd<Lhs, Rhs, Output = TyTrue>
```

* **Pros**: Decouples logic from the data types.

* **Cons**: Verbose.
* **Status**: Generally avoided, but reserved for future "Predicate Trait" patterns where we need to validate properties without consuming the values.

#### Pattern C: Result-Based Dispatch (Unused)

The expected result (`Output`) acts as the subject.

```rust
where IsTrue: TyAnd<Lhs, Rhs>
```

* **Cons**: Counter-intuitive readability.

* **Status**: Documented as a theoretical possibility but not used.

**Note**: We do not provide "Shortcut Traits" (e.g., `TyAndTrue<Lhs, Rhs>`) to strictly check for specific outputs. Wrapping every possible output condition leads to trait explosion. If needed, we will solve this via macros.

### 5.2 Public API Design Models

We explored how to expose these traits to the user:

#### Model 1: Function-style Type Aliases

```rust
pub type And<L, R> = <L as TyAnd<R>>::Output;
```

* **Pros**: Simple, intuitive, looks like a function.

* **Cons**: **Trait Bound Hell**. Usage propagates bounds deeper and deeper (e.g., `Or<A, And<B, C>>` requires satisfying bounds for `A`, `B`, and `C` explicitly).
* **Status**: Used for internal aliases, but problematic for complex nested logic.

#### Model 2: The `Evaluable` Pattern (Adopted)

This is the architecture described in Section 3 ("The CPU").

1. Define a struct representing the AST node (e.g., `struct EAnd<L, R>`).
2. Implement `Evaluable` for this struct.
3. Encapsulate all trait bounds and calculation logic *inside* the `Evaluable` implementation.

```rust
impl<L, R> Evaluable for EAnd<L, R>
where
    L: Evaluable,
    R: Evaluable,
    Evaluator<L>: TyAnd<Evaluator<R>>,
{
    type Output = <Evaluator<L> as TyAnd<Evaluator<R>>>::Output;
}
```

* **Pros**: The user only sees `where T: Evaluable`. exact bounds are hidden.

* **Status**: The standard way Typelude exposes functionality.

#### Model 3: Predicate Traits

(Future work) - Specialized traits for asserting properties of types.
