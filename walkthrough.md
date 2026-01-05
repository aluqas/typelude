# Walkthrough: Refactor Typelude Imports

## Objective
Refactor the `typelude` ecosystem to resolve import errors resulting from the restructuring of the `eval` module and the separation of core traits. The goal was to ensure all crates (`typelude-std`, `typelude-vm`, `typelude`, `typelude-macros`) correctly import `Eval`, `Evaluate`, and `App` from `typelude_core`.

## Changes

### 1. `typelude-core`
- Confirmed `Eval`, `Evaluate`, `Apply`, `App`, and `Sealed` are defined in and exported from `typelude_core`.
- Added `ELit` (Literal Wrapper) to `typelude-core` as it serves as a fundamental evaluation barrier.

### 2. `typelude-std`
- Removed local `eval` module declarations and re-exports that caused conflicts or circular dependencies.
- Updated all submodules (`expr`, `std::ops`, `std::primitives`, `lambda`, etc.) to import `Eval`, `Evaluate`, and `ELit` directly from `typelude_core`.
- Fixed `macros.rs` to reference `$crate::typelude_core::*`.

### 3. `typelude-macros`
- Updated the `def_op!` proc-macro implementation (`src/def_op.rs`) to generate code that imports traits from `typelude_core` instead of `crate::eval`.

### 4. `typelude-vm`
- Updated machine execution, state, and tracing modules to import `Eval`, `Evaluate`, `Sealed`, and `App` from `typelude_core`.
- Resolved `TyNil`/`TyArray` usage via `typelude_std`.

### 5. `typelude` (Meta-crate)
- Updated `lib.rs` to export `Eval` and `Evaluate` from `typelude_core`.
- Removed deprecated `eval` exports from `typelude_std`.
- Updated integration tests (`tests/program_macro_tests.rs`) to use the new top-level exports.

## Verification
- Ran `cargo check --workspace` to ensure all crates compile.
- Ran `cargo test --workspace` to ensure all tests pass (including doc tests and integration tests).

## Result
The refactoring is complete. The dependency graph is cleaner, with `typelude_core` serving as the source of truth for evaluation traits, and `typelude_std`/`typelude_vm` consuming them.
