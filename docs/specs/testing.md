//! **Testing Guidelines**
//!
//! This document outlines the testing strategy and guidelines for the project.
//!
//! # Testing Strategy
//!
//! We follow the standard Rust testing conventions with a focus on type-level programming specifics.
//!
//! ## 1. Unit Tests (`#[cfg(test)] mod tests`)
//!
//! - **Location**: Inside the source file being tested, at the bottom.
//! - **Scope**:
//!     - Test individual type definitions and traits.
//!     - Verify `Evaluable` implementations for specific types.
//!     - Test internal helper traits and logic.
//! - **Tools**:
//!     - Use `static_assertions::assert_type_eq_all!` to verify type equality.
//!     - Use `static_assertions::assert_type_ne_all!` to verify type inequality.
//!     - Use `crate::eval::Evaluator` to compute the result of an expression.
//!
//! ## 2. Integration Tests (`tests/`)
//!
//! - **Location**: In the `tests/` directory at the project root.
//! - **Scope**:
//!     - Test the interaction between multiple modules (e.g., `machine` and `std`).
//!     - Test complex programs defined using the `program!` macro.
//!     - Verify end-to-end behavior of the stack machine.
//! - **Naming**:
//!     - `tests/machine_tests.rs`: Tests for machine execution and instruction sets.
//!     - `tests/std_tests.rs`: Tests for standard library features (if they span multiple modules).
//!     - `tests/macro_tests.rs`: Tests for macros (optional, often better as unit tests if they rely on internal structure).
//!
//! ## 3. Testing Type-Level Logic
//!
//! Since this is a type-level library, "running" a test often means compiling it.
//! However, we want to assert outcomes.
//!
//! - **Success**: `assert_type_eq_all!(Evaluator<Expr>, ExpectedResult);`
//! - **Failure**: Since we can't easily assert "compilation failure" in standard `cargo test` without trybuild (which is slow), we focus on positive assertions.
//!     - *Note*: If a type constraint is violated, the test will fail to compile. This is a valid "test failure" but hard to catch in CI unless expected.
//!     - For now, assume if it compiles and the output type matches, it passes.
//!
//! ## 4. Best Practices
//!
//! - **Descriptive Names**: Test functions should be named `test_<feature>_<condition>`.
//! - **Comments**: Explain *what* the type-level program is doing, as S-expressions can be hard to read.
//! - **Isolation**: Ensure tests don't depend on global state (though type-level state is immutable/functional, macro-defined vars are global scope).
//!     - *Warning*: `define_vars!` creates global types. Reuse standard variable names (`x`, `y`, `z`) carefully or scope them if possible.
//!
//! # Example
//!
//! ```rust
//! #[test]
//! fn test_addition() {
//!     type Result = Evaluator<EAdd<U1, U2>>;
//!     assert_type_eq_all!(Result, U3);
//! }
//!```
