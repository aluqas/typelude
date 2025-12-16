# Typelude - Rust Type Utilities

Typelude provides advanced logical operations and utilities for Rust's type system.

## Documentation

- [Architecture](./docs/ARCHITECTURE.md) - Project structure and typology.
- [Style Guide](./docs/STYLE_GUIDE.md) - Naming conventions and coding standards.
- [Philosophy](./docs/PHILOSOPHY_AND_THEORY.md) - Theoretical background.

## Features

- **Type-Level Values**: `TyTrue`, `TyFalse`, `TyArray`.
- **Expressions**: AST-based evaluation (`EAdd`, `EIf`).
- **Stack Machine**: Type-level execution environment.

## Architecture

The library follows a 6-layer typology:

1. **Values**: Concrete data structures (`TyArray`).
2. **Capabilities**: Traits describing actions (`Len`, `Get`).
3. **OpCodes**: Instruction markers (`FAdd`).
4. **Backends**: Recursive helpers (`MapHelper`).
5. **Evaluator**: Unified `Evaluable` interface.
6. **Aliases**: User-facing type aliases (`EAdd`).

Please refer to `docs/STYLE_GUIDE.md` for contribution guidelines and naming conventions.
