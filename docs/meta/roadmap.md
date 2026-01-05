---
id: meta-roadmap
title: "Typelude Roadmap"
status: current
created: 2025-01-01
---

# Typelude Roadmap

**Typelude** is a research project dedicated to turning the Rust compiler's type system into a fully-fledged runtime environment.

This roadmap outlines the path from "Theoretical Type Puzzle" to "Industrial Grade Metaprogramming Engine".

### Reference

* [Theoretical Foundation](philosophy.md)
* [System Architecture](../specs/architecture.md)

---

## Phase 0: Genesis (✅ Complete)

**Goal:** Prove that a Turing-complete Stack Machine concept is viable within the Trait Solver.

* [x] **Core Machine**: Implement `MachineState`, `Stack`, and `Evaluator`.
* [x] **Basic Arithmetic**: `Add`, `Sub` on PEANO numbers.
  * [x] Implemented Inductive addition for `Z` and `S<N>`.
* [x] **Control Flow**: `If`, `While` using lazy type evaluation.
  * [x] Solved infinite recursion issues using `EFunction` and `EApply` pattern.
* [x] **Local Variables**: `Let`, `Set`, `Get` for managing register state.
* [x] **Proof of Concept**: Successfully calculate `Fibonacci(10)` purely at compile time.

---

## Phase 1: The Industrial Revolution (🚧 Current Focus)

**Goal:** Abstract away the boilerplate. Make the library usable by humans, not just wizards.

**Core Tasks:**
* [ ] **Instruction Definition Macro** (`def_instruction!`):
  * **Spec**: `def_instruction!(OpName: Stack[A, B] -> Stack[C] where ...)`
  * **Goal**: Auto-generate the `impl Execute` and `impl RunStep` boilerplate.
  * **Metric**: Reduce new opcode definition code size by 90%.
* [ ] **Control Flow Macros** (`def_dispatch!`):
  * Simplify the creation of conditional logic (If/Else) which currently requires manual lazy-eval setups.
* [ ] **Data Structure Library**:
  * Implement `OpList` (Cons/Car/Cdr) operations for the user.
  * Implement `OpPair` (Tuple) operations.
* [ ] **Testing Infrastructure**:
  * Integrate `trybuild` for negative testing (verifying that invalid programs fail to compile with *readable* errors).
  * Integrate `cargo-expand` usage validation.

**Technical Challenges:**

* **Macro Hygiene**: Ensuring the generated `impl` blocks don't conflict with user defined types.
* **Error Messages**: When a macro-generated constraint fails, the error message often points to the macro definition, not the usage site. We need to work on `#[diagnostic::on_unimplemented]` attributes.

---

## Phase 2: Turing Completeness++ (Planned)

**Goal:** Implement the complex features required for "Real" software.

**Core Tasks:**
* [ ] **Function Calls (`Call` / `Ret`)**:
  * Implement a "Call Stack" within the Type System.
  * Allow reusable procedures (currently we only have `GOTO` loops).
  * **Isolation**: Ensure local variables in the callee do not leak into the caller.
* [ ] **Heap Memory (`Alloc` / `Load` / `Store`)**:
  * Implement a Global Memory Map separate from the operand stack.
  * Enable pointer-like behavior (referencing memory index `U5`).
* [ ] **Advanced Data Types**:
  * **Signed Integers**: Two's complement math on types (e.g., `PInt<U5>`, `NInt<U2>`).
  * **Strings**: Type-level character arrays and manipulation (concatenation, splicing).
  * **Fixed-Point Math**: Representing decimals/fractions.

**Technical Challenges:**

* **Recursion Limits**: Deep call stacks will hit the default instantiation depth of `rustc`. Users will need `#![recursion_limit = "512"]` or higher.
* **Memory Efficiency**: The "Heap" (an HList) grows linearly. Accessing index `N` is O(N) compile-time complexity. We might need tree-based structures (Type-Level Binary Tree) for O(log N) access.

---

## Phase 3: The Renaissance (Future)

**Goal:** Aesthetics, User Interface, and "Magical" features.

**Core Tasks:**
* [ ] **ASCII Art Panic Messages**:
  * Exploit `const_panic` to render beautiful error screens when a type program crashes.
  * "The Blue Screen of Death" but in your compiler terminal.
* [ ] **Time-Traveling Debugger**:
  * Since the type system is functional, we can return the *entire history* of execution in the result type.
  * Create a tool to inspect `State<T=0>` vs `State<T=100>`.
* [ ] **The Transpiler (`#[type_lift]`)**:
  * A procedural macro that reads *normal* Rust code function and converts it into Typelude assembly automatically.
  * Write `fn add(a, b) { a + b }` -> Get `Cons<OpPush<A>, Cons<OpPush<B>, Cons<OpAdd...>>>`.

**Technical Challenges:**

* **AST Parsing**: Writing a full Rust parser in a proc-macro is heavy (syn does this, but mapping it to our opcodes is non-trivial).
* **Terminal Formatting**: `const_eval` does not really support string formatting easily.

---

## Phase 4: The Singularity (Visionary)

**Goal:** If we can run a stack machine, we can run... anything.

**Core Tasks:**
* [ ] **Type-Level WASM Runtime**:
  * A translator that converts WebAssembly bytecode into Typelude Instructions.
  * Run WASM modules at compile time.
* [ ] **Type-Level Raytracer**:
  * Render images to the terminal (or generating BMP files) purely via type expansion.
  * Requires implementing vector math and collision logic.
* [ ] **Static Analysis Engine**:
  * Use the VM to prove complex invariants about runtime code (e.g., verifying state machine transitions).

**Technical Challenges:**

* **Compilation Time**: Rendering a raytraced image might take 24 hours of compilation. Parallelism in `rustc` is improving, but deep trait chains are mostly single-threaded.
