# Typelude Refactoring Proposal: Abstraction & Separation

This document outlines a proposal for refactoring the `typelude` ecosystem to clarify the separation between computation models (Abstraction) and concrete implementations, and to resolve dependency and architectural ambiguities.

## 1. Architectural Restructuring (Crate Split)

To resolve circular dependencies and enforce a strict hierarchy, we propose splitting `typelude-core` into two distinct crates: **`typelude-kernel`** and **`typelude-core`** (or `typelude-std`).

### Current Structure (Ambiguous)
*   `typelude-core`
    *   `eval/` (Core Engine)
    *   `std/` (Primitives & Traits)
    *   `lambda/` (Lambda Calculus - distinct model)

### Proposed Structure (Hierarchical)

#### 1. `typelude-kernel` (The Engine)
*   **Purpose:** Pure evaluation logic. Minimal dependencies.
*   **Contents:**
    *   `Eval` Trait (The "CPU").
    *   `Apply` Trait (The "Instruction Step").
    *   `App<Op, Args>` (The Unified AST / "Instruction Format").
    *   `Bridge` (Utilities to bridge Lambda/App worlds).
    *   `Identity` / `TIdentity` (Basic type identity).

#### 2. `typelude-core` (The Standard Library / Models)
*   **Purpose:** Defines the "Type Class" abstractions and provides standard implementations.
*   **Dependencies:** `typelude-kernel`, `typenum`.
*   **Contents:**
    *   **Traits (The Axis):** `TypeNat`, `TypeBool`, `TypeList` (or `Cons`).
    *   **Implementations:**
        *   `int.rs`: Adapters for `typenum`.
        *   `bool.rs`: `TyTrue`, `TyFalse`.
        *   `array.rs`: `TyArray` (standard list).
    *   **Operators:** `OpAdd`, `OpAnd`, `OpIf` (Implemented generically over the Traits).

#### 3. `typelude-lambda` (The Alternative Model)
*   **Purpose:** Pure Lambda Calculus implementation (Church/Scott encodings).
*   **Dependencies:** `typelude-kernel`.
*   **Note:** Can now depend on `kernel` without pulling in the heavy `std` primitives if not needed, or vice-versa.

---

## 2. Unifying the Execution Model (App vs. AST)

Currently, there is a mix of specific AST structs (`EAdd`, `EIf`) and the generic `App<Op, Args>`.

### The Ambiguity
*   `EAdd<A, B>` is a struct.
*   `App<OpAdd, (A, B)>` is a generic application.
*   Having both creates confusion: "Which one should I implement `Eval` for?"

### The Proposal: "Internal App, External AST"
We will unify the **internal representation** to `App` (or a standardized form) while keeping the **external API** readable via Macros.

1.  **The Single Truth:** The Evaluator primarily understands `App<Op, Args>`.
2.  **The Wrapper:** Specific structs like `EAdd` become type aliases or newtypes around `App`.
3.  **The Macro (`def_op!`):**
    A macro will automate the generation of these wrappers.

```rust
// Conceptual Macro Usage
def_op! {
    // Defines struct OpAdd;
    // Defines type EAdd<L, R> = App<OpAdd, (L, R)>;
    // Implements Apply for OpAdd where ...
    name: OpAdd,
    args: (L, R),
    alias: EAdd,
    impl: |l, r| { <L as TypeAdd<R>>::Output } // Logic uses the Abstract Trait
}
```

This enforces that **all** operations go through the `Apply` -> `Eval` pipeline, removing "magic" implementation details from AST structs.

---

## 3. Abstraction Axis: Traits vs. Types

We will formalize the "Type Class" pattern to separate the *concept* of a type from its *implementation*.

### The Axis (Traits)
These traits define what operations are supported.
*   **`TypeBool`:** Requires `TypeAnd`, `TypeOr`, `TypeNot`.
*   **`TypeNat`:** Requires `TypeAdd`, `TypeSub`, etc. (already partially done).

### The Implementation (structs)
*   **`typenum`:** Implements `TypeNat`.
*   **`TyTrue`/`TyFalse`:** Implement `TypeBool`.
*   **Future/User Types:** A user can create `MyBool` and just implement `TypeBool` to work with `OpIf`.

**Refactoring `TyTrue` / `TyFalse`:**
As per your suggestion, `TyTrue` and `TyFalse` could technically be treated as zero-sized marker types that implement a `TypeBool` trait, rather than just being "the" boolean types.
*   **Goal:** `OpIf<Cond, Then, Else>` should work for *any* `C: TypeBool`, not just `TyTrue/TyFalse`.

---

## 4. Nightly vs. Stable Strategy

We will use Cargo features to manage stability.

*   **`Cargo.toml`**:
    ```toml
    [features]
    default = ["nightly"]
    nightly = [] # Enables specialized impls, generic_const_exprs
    ```

*   **Strategy:**
    *   **Stable:** Fallback implementations.
        *   `OpEq`: Implemented specific pairs (e.g., `(TyTrue, TyTrue)`, `(U0, U0)`). Might be incomplete but "safe".
    *   **Nightly:** Full power.
        *   `OpEq`: Generic implementation using specialization or const generics to compare *any* two types.

## 5. Next Steps (Plan)

1.  **Extract `typelude-kernel`**: Move `eval` module to a new crate.
2.  **Refactor `typelude-core`**: Update imports to use `kernel`.
3.  **Macro Unification**: Update `def_op!` (or similar) to enforce the `App` pattern and remove ad-hoc `Eval` impls on AST structs.
4.  **Trait Formalization**: Review `std/traits.rs` and ensure Primitives implement them cleanly.
