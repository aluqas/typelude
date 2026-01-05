---
id: rfc-0002
title: "Lambda-Based Control Flow Unification"
status: draft
created: 2026-01-05
author:
---

# RFC-0002: Lambda-Based Control Flow Unification

## Summary

Refactor control flow primitives (`If`, `While`, `For`) to be grounded in pure lambda calculus theory from `lambda/`, with practical adapters living in `std/`.

## Design Principle

> **`lambda/` = Theoretical Foundation (Pure)**
> **`std/` = Practical Application (Adapted)**

The `lambda/` module must remain a **pure lambda calculus implementation**:

- Church booleans, numerals, lists
- Combinators (S, K, I, Y)
- Monads as pure functional constructs

All adaptations to Rust-specific types (`TyTrue`, `typenum`, etc.) belong in `std/`.

---

## Current State

### lambda/ (Pure Theory)

| Component         | Location                 | Role             |
| ----------------- | ------------------------ | ---------------- |
| `LTrue`, `LFalse` | `lambda/church/bool.rs`  | Church booleans  |
| `LIf`, `LPureIf`  | `lambda/church/bool.rs`  | Pure conditional |
| `LState`, `LBind` | `lambda/monads/state.rs` | State monad      |
| `LFix`            | `lambda/fix.rs`          | Y-combinator     |

### std/ (Practical)

| Component           | Location                  | Role                        |
| ------------------- | ------------------------- | --------------------------- |
| `EIf`               | `expr.rs`                 | `TyTrue`/`TyFalse` dispatch |
| `EWhile`            | `expr.rs`                 | Apply-based loop            |
| `TyTrue`, `TyFalse` | `data/primitives/bool.rs` | Type-level booleans         |

### Problems

1. `expr.rs` duplicates lambda logic without grounding in theory
2. No clear bridge from practical types to lambda foundation
3. std functions (`Filter`) use `EIf` directly, missing composability

---

## Proposed Architecture

```
┌─────────────────────────────────────────────┐
│                  User Code                  │
└──────────────────────┬──────────────────────┘
                       │ uses
┌──────────────────────▼──────────────────────┐
│              std/control.rs                 │
│  ┌────────────────────────────────────────┐ │
│  │ EIf, EWhile, EFor (practical API)      │ │
│  │ ToChurch, FromChurch (adapters)        │ │
│  └───────────────────┬────────────────────┘ │
└──────────────────────┼──────────────────────┘
                       │ delegates to
┌──────────────────────▼──────────────────────┐
│              lambda/ (pure)                 │
│  ┌────────────────────────────────────────┐ │
│  │ LIf, LWhile (pure combinators)         │ │
│  │ LTrue, LFalse (Church booleans)        │ │
│  │ LState, LBind (monads)                 │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

---

## Detailed Design

### Part 1: Extend lambda/ with Pure Combinators

Add missing pure constructs **without any Rust-specific types**:

```rust
// lambda/church/control.rs [NEW]

/// Pure WhileM combinator
/// whileM :: (s -> Bool) -> (s -> s) -> s -> s
pub struct LWhile;

impl<Pred, Body, State> Lambda for LApp<LApp<LApp<LWhile, Pred>, Body>, State>
where
    // pred(state) -> LTrue/LFalse
    LApp<Pred, State>: Lambda,
    // LIf (pred state) (whileM pred body (body state)) state
    LPureIf<
        <LApp<Pred, State> as Lambda>::Output,
        LApp<LApp<LApp<LWhile, Pred>, Body>, <LApp<Body, State> as Lambda>::Output>,
        State,
    >: Lambda,
{
    type Output = ...;
}
```

### Part 2: Bridge Layer in std/

Create `std/control.rs` with adapters:

```rust
// std/control.rs [NEW]

use crate::lambda::church::{LTrue, LFalse, LIf, LPureIf, LWhile};
use crate::data::primitives::bool::{TyTrue, TyFalse};

/// Convert TyBool to Church Boolean
pub trait ToChurch {
    type Church;
}

impl ToChurch for TyTrue { type Church = LTrue; }
impl ToChurch for TyFalse { type Church = LFalse; }

/// Practical If: wraps LPureIf with ToChurch
pub type EIf<Cond, Then, Else> = LPureIf<
    <Evaluate<Cond> as ToChurch>::Church,
    Then,
    Else,
>;

/// Practical While: wraps LWhile with adapters
pub type EWhile<Pred, Step, State> = LApp<LApp<LApp<
    LWhile,
    ChurchifyPred<Pred>
>, Step>, State>;
```

### Part 3: Migrate expr.rs

```rust
// expr.rs [DEPRECATED]

#[deprecated(note = "Use std::control::EIf")]
pub use crate::std::control::EIf;

#[deprecated(note = "Use std::control::EWhile")]
pub use crate::std::control::EWhile;
```

---

## Module Structure

```
lambda/                          # PURE THEORY
├── church/
│   ├── bool.rs                  # LTrue, LFalse, LIf
│   ├── nat.rs                   # Church numerals
│   └── control.rs               # [NEW] LWhile, LFor (pure)
├── monads/
│   └── state.rs                 # LState, LBind, LGet, LPut
└── fix.rs                       # Y-combinator

std/                             # PRACTICAL APPLICATION
├── control.rs                   # [NEW] EIf, EWhile, ToChurch
├── primitives/
│   ├── bool.rs                  # TyTrue, TyFalse
│   └── array.rs                 # uses std::control::EIf
└── ops.rs

expr.rs                          # [DEPRECATED] -> re-exports std/control
```

---

## Migration Path

| Phase | Scope                                             | Location                   |
| ----- | ------------------------------------------------- | -------------------------- |
| 1     | Add `LWhile` pure combinator                      | `lambda/church/control.rs` |
| 2     | Create `std/control.rs` with adapters             | `std/control.rs`           |
| 3     | Deprecate `expr.rs`, re-export from `std/control` | `expr.rs`                  |
| 4     | Update `FilterHelper` etc. to use new imports     | `std/primitives/array.rs`  |

---

## Benefits

1. **Theoretical Purity**: `lambda/` remains a clean reference implementation
2. **Separation of Concerns**: Rust adaptations isolated in `std/`
3. **Provability**: Pure lambda constructs are easier to reason about
4. **Flexibility**: Can swap adapter strategies without touching theory

---

## Unresolved Questions

1. **Naming**: `std/control.rs` vs `std/ops/control.rs`?
2. **LWhile Termination**: How to prove termination at type level?
3. **Effect Integration**: How does this interact with State monad usage?

---

## Part 4: Macro-Based Design Pattern Enforcement

### 4.1 Existing Macro Patterns

| Macro                    | Purpose           | Generated Items                                  |
| ------------------------ | ----------------- | ------------------------------------------------ |
| `define_arith_op!`       | Binary arithmetic | `OpX`, `EX<Lhs, Rhs>`, `impl Eval`, `impl Apply` |
| `define_logic_op!`       | Binary logic      | `OpX`, `EX<Lhs, Rhs>`, `impl Eval`, `impl Apply` |
| `define_unary_logic_op!` | Unary logic       | `OpX`, `EX<Val>`, `impl Eval`, `impl Apply`      |
| `def_op!` (proc-macro)   | Generic ops       | `OpX`, `EX<...>`, `impl Eval`, `impl Apply`      |

### 4.2 Proposed: `define_control_op!`

New macro for control flow operations:

```rust
#[macro_export]
macro_rules! define_control_op {
    // If-style: Cond + two branches
    (if: $op_name:ident, $doc:literal) => {
        $crate::paste::paste! {
            #[doc = $doc]
            pub struct [<Op $op_name>];
            impl $crate::typelude_core::Sealed for [<Op $op_name>] {}

            pub struct [<E $op_name>]<Cond, Then, Else>(
                std::marker::PhantomData<(Cond, Then, Else)>
            );

            impl<Cond, Then, Else> $crate::typelude_core::Eval
                for [<E $op_name>]<Cond, Then, Else>
            where
                Cond: $crate::typelude_core::Eval,
                $crate::typelude_core::Evaluate<Cond>: ToChurch,
                // Delegate to pure lambda LPureIf
                $crate::lambda::church::LPureIf<
                    <$crate::typelude_core::Evaluate<Cond> as ToChurch>::Church,
                    Then,
                    Else,
                >: $crate::typelude_core::Eval,
            {
                type Output = /* ... */;
            }
        }
    };

    // While-style: Pred + Step + State
    (while: $op_name:ident, $doc:literal) => {
        // Similar pattern, delegating to LWhile
    };
}
```

### 4.3 Design Pattern Invariants

The macros enforce:

1. **Naming Convention**: `Op*` (operator), `E*` (expression)
2. **Sealed Trait**: All `Op*` implement `Sealed`
3. **Eval Pattern**: `E*` structs implement `Eval` with `Evaluate<>` wrapping
4. **Apply Bridge**: `Op*` implements `Apply<Args> -> E*<Args>`
5. **PhantomData**: All E* use `PhantomData` for ZST

---

## Part 5: State Implementation Audit

### 5.1 Audit Scope

| Module                    | Implementation     | State Pattern                  | Status       |
| ------------------------- | ------------------ | ------------------------------ | ------------ |
| `std/ops/arith.rs`        | `define_arith_op!` | Stateless                      | ✅ Sound      |
| `std/ops/logic.rs`        | `define_logic_op!` | Stateless                      | ✅ Sound      |
| `std/ops/cmp.rs`          | `def_op!`          | Stateless                      | ✅ Sound      |
| `std/primitives/array.rs` | Manual             | Stateless (recursion via Tail) | ⚠️ Review     |
| `std/primitives/bool.rs`  | Manual             | Stateless                      | ✅ Sound      |
| `expr.rs` (`EIf`)         | Manual             | Stateless dispatch             | ⚠️ To migrate |
| `expr.rs` (`EWhile`)      | Manual             | Apply-threaded state           | ⚠️ To migrate |

### 5.2 Findings

#### ✅ Sound: Macro-Generated Operations

```rust
// std/ops/arith.rs - Clean pattern
define_arith_op!(Add, TypeAdd, "Addition: A + B");
```

- No state threading
- Pure evaluation via trait bounds
- Consistent pattern

#### ⚠️ Review: Array Higher-Order Functions

```rust
// EMap, EFilter, EFold - Use helper traits
impl<Op, List> Eval for EMap<Op, List>
where
    List: Eval,
    Evaluate<List>: MapHelper<Op>,  // State hidden in MapHelper
{
    type Output = <Evaluate<List> as MapHelper<Op>>::Output;
}
```

**Issue**: State (list traversal) is implicit in `MapHelper` recursion.
**Recommendation**: Document as "fold-based" pattern; no action needed.

#### ⚠️ To Migrate: expr.rs Control Flow

```rust
// EWhile - Ad-hoc state threading
impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    Step: Apply<Evaluate<State>>,
    // ... WhileHelper dispatch
```

**Issue**: Does not ground in lambda theory.
**Action**: Migrate per RFC-0002 Part 2.

### 5.3 Audit Conclusion

| Category       | Count                          | Action             |
| -------------- | ------------------------------ | ------------------ |
| Sound (macro)  | 12+ ops                        | None               |
| Sound (manual) | ~10                            | None               |
| To Migrate     | 2 (`EIf`, `EWhile`)            | RFC-0002 Phase 2-3 |
| Review         | 3 (`EMap`, `EFilter`, `EFold`) | Document pattern   |

---

## Appendix: File-by-File Changes

| File                       | Current         | Proposed                    |
| -------------------------- | --------------- | --------------------------- |
| `lambda/church/control.rs` | (new)           | `LWhile`, `LFor`            |
| `std/control.rs`           | (new)           | `EIf`, `EWhile`, `ToChurch` |
| `std/macros.rs`            | arith/logic     | + `define_control_op!`      |
| `expr.rs`                  | `EIf`, `EWhile` | deprecated re-exports       |
| `std/primitives/array.rs`  | `use expr::EIf` | `use std::control::EIf`     |

---

## References

- [Church Encoding (Wikipedia)](https://en.wikipedia.org/wiki/Church_encoding)
- [typelude Philosophy](../meta/philosophy.md)
- Current macros: `macros.rs`, `def_op.rs`
