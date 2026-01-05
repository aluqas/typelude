//! **Core Traits**
//!
//! Core trait definitions for `typelude`.
//!
//! This module defines the "Type Classes" that abstraction layers (like `std`) use.
//! Implementations are provided in their respective modules (`int`, `bool`, `array`).
//!
//! ## Interoperability Pattern
//!
//! To use external types (like `typenum` integers or custom structs) with `typelude` operations,
//! you must implement the relevant "Type Class" traits for them.
//!
//! For example, to use a custom boolean type `MyBool` with `OpAnd`, `OpIf`, etc.:
//! 1. Implement `Eval` for `MyBool` (usually identity).
//! 2. Implement `TypeBool` for `MyBool`.
//!
//! This "Adapter Pattern" allows the core logic to remain agnostic of the underlying concrete types.

use typelude_core::{ELit, Eval};

/// Trait representing a function
///
/// Represents a transformation from type to type, used in `EWhile` etc.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid type-level function for argument `{Arg}`",
    label = "this type cannot be applied to `{Arg}`",
    note = "ensure `{Self}` implements `TyFn<{Arg}>`"
)]
pub trait TyFn<Arg> {
    type Output;
}

// Automatically unwrap ELit
#[diagnostic::do_not_recommend]
impl<F, Arg> TyFn<ELit<Arg>> for F
where
    F: TyFn<Arg>,
{
    type Output = <F as TyFn<Arg>>::Output;
}

// =============================================================================
// Math Traits (Arithmetic)
// =============================================================================

/// Marker trait for Natural Numbers (Unsigned Integers).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Natural Number",
    label = "not a TypeNat",
    note = "implement `TypeNat` for `{Self}` to use it in arithmetic operations"
)]
pub trait TypeNat: Eval {
    // We can add associated types here if needed, e.g., Next, Prev.
}

/// Marker trait for Integers (Signed).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Integer",
    label = "not a TypeInt"
)]
pub trait TypeInt: Eval {
    // Sign, Abs, etc.
}

/// Type-level Addition: `Lhs + Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot add `{Rhs}` to `{Self}`",
    label = "addition not implemented",
    note = "ensure `{Self}` implements `TypeAdd<{Rhs}>`"
)]
pub trait TypeAdd<Rhs> {
    type Output;
}

/// Type-level Subtraction: `Lhs - Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot subtract `{Rhs}` from `{Self}`",
    label = "subtraction not implemented"
)]
pub trait TypeSub<Rhs> {
    type Output;
}

/// Type-level Multiplication: `Lhs * Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot multiply `{Self}` by `{Rhs}`",
    label = "multiplication not implemented"
)]
pub trait TypeMul<Rhs> {
    type Output;
}

/// Type-level Division: `Lhs / Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot divide `{Self}` by `{Rhs}`",
    label = "division not implemented"
)]
pub trait TypeDiv<Rhs> {
    type Output;
}

/// Type-level Remainder: `Lhs % Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot calculate remainder of `{Self}` / `{Rhs}`",
    label = "remainder not implemented"
)]
pub trait TypeRem<Rhs> {
    type Output;
}

/// Type-level Exponentiation: `Lhs ^ Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot raise `{Self}` to the power of `{Rhs}`",
    label = "exponentiation not implemented"
)]
pub trait TypePow<Rhs> {
    type Output;
}

// =============================================================================
// Logic Traits (Boolean)
// =============================================================================

/// Type-level Boolean Logic.
///
/// Implementors: `TyTrue`, `TyFalse`, `B0`, `B1`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Boolean",
    label = "not a TypeBool",
    note = "implement `TypeBool` for `{Self}` to use it in logical operations"
)]
pub trait TypeBool: Eval {
    /// Result of `!Self`
    type Not;
    /// Result of `Self && Rhs`
    type And<Rhs: TypeBool>;
    /// Result of `Self || Rhs`
    type Or<Rhs: TypeBool>;
    /// Result of `Self ^ Rhs`
    type Xor<Rhs: TypeBool>;
    /// Result of `!(Self && Rhs)`
    type Nand<Rhs: TypeBool>;
    /// Result of `!(Self || Rhs)`
    type Nor<Rhs: TypeBool>;
    /// Result of `!(Self ^ Rhs)`
    type Xnor<Rhs: TypeBool>;
}

// =============================================================================
// Collection Traits (List/Array)
// =============================================================================

/// Type-level List (Cons List).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level List",
    label = "not a TypeList",
    note = "implement `TypeList` for `{Self}` to use it in list operations like `Map`, `Filter`, `Fold`"
)]
pub trait TypeList: Eval {
    /// The type of the head element.
    type Head;
    /// The type of the tail (must also be a TypeList, or Nil).
    type Tail;
    /// Prepend a new element to this list.
    type Cons<NewHead>;
}
