# Code Style & Conventions

This document outlines the coding standards, naming conventions, and documentation guidelines for the `typeutils` project. Adherence to these rules ensures consistency, readability, and maintainability across the codebase.

## 1. Documentation Guidelines

* **Language**: All documentation, code comments, and commit messages must be in **English**.
* **Rustdoc**: Use standard Rustdoc syntax.
  * Use `///` for documenting items (structs, traits, functions).
  * Use `//!` for module-level documentation at the top of files.
* **Cleanliness**:
  * Avoid visual clutter and "decoration" comments.
  * **Do not** use large ASCII art separators or banners (e.g., `// ==================`).
  * Use Markdown headers (`#`, `##`, `###`) within doc comments to structure information if necessary.

## 2. Naming Conventions

We prioritize **Rust-idiomatic** names that convey meaning, rather than cryptic single-letter variables typical in some type-level libraries. This improves error message readability and developer experience.

### Generic Type Parameters

Standardize generic type parameters according to their role:

| Category           | Generic Names          | Example                      | Notes                                                               |
| :----------------- | :--------------------- | :--------------------------- | :------------------------------------------------------------------ |
| **Binary Ops**     | `Lhs`, `Rhs`           | `impl Add<Rhs> for Lhs`      | Replaces `A, B`. Consistent with `std::ops`.                        |
| **Unary Ops**      | `Val`                  | `impl Not for Val`           | Replaces `A` or `T`.                                                |
| **Functions**      | `Arg`                  | `EFunction<Arg>`             | Input argument for a type-level function.                           |
| **Conditionals**   | `Cond`, `Then`, `Else` | `EIf<Cond, Then, Else>`      | Clear distinction of branches.                                      |
| **Collections**    | `Head`, `Tail`         | `struct TyArray<Head, Tail>` | Used in structure definitions.                                      |
| **Implementation** | `H`, `T`               | `impl<H, T> Head for ...`    | Shortened in `impl` blocks to avoid shadowing traits `Head`/`Tail`. |
| **Indices**        | `Idx`                  | `Get<Idx>`                   | Represents a numeric index (De Bruijn or Array).                    |
| **Elements**       | `Elem`                 | `Contains<Elem>`             | An item being searched for or manipulated.                          |
| **Machine**        | `Prog`, `Stack`        | `MachineState<..., Prog>`    | Context objects in the stack machine.                               |

### Type Prefixes & Architecture

The project follows a strict 6-layer typology, reflected in naming:

1. **Values (`Ty*`)**: Concrete data structures.
    * Prefix: `Ty` (e.g., `TyArray`, `TyTrue`, `TyNil`).
    * *Note: Using full names (`Array`) vs prefixes (`TyArray`) vs short prefixes (`TArr`) is a subject of ongoing consideration for compiler error optimization.*
2. **Expressions (`E*`)**: AST nodes for the evaluator.
    * Prefix: `E` (e.g., `EAdd`, `EIf`, `EApply`).
3. **OpCodes (`F*`)**: Zero-sized markers representing functions/verbs.
    * Prefix: `F` (e.g., `FAdd`, `FMap`).
4. **Capabilities**: Traits describing actions.
    * Naming: Verbs or Nouns (e.g., `Add`, `Cons`, `Len`).
5. **Backends**: Internal recursive implementations.
    * Suffix: `Helper` (e.g., `MapHelper`).
    * Visibility: Generally `#[doc(hidden)]`.

## 3. Formatting

* Always run `cargo fmt` before committing.
* Organize imports into groups (std, internal modules, external crates).

## 4. Testing

* **Doc Tests**: Ensure public examples in documentation compile and run (`cargo test --doc`).
* **Unit Tests**: Place in a `mod tests` block at the bottom of the file.
* **Run All**: Frequent `cargo test` execution is required to catch regression in generic constraints.
