//! Core traits used by `std` adapters.
//!
//! `TyFn` is canonically exposed at `typelude_std::core::TyFn` and re-exported
//! here.

pub use crate::core::TyFn;

/// Marker trait for Natural Numbers (Unsigned Integers).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Natural Number",
    label = "not a Nat",
    note = "implement `Nat` for `{Self}` to use it in arithmetic operations"
)]
pub trait Nat {}

/// Marker trait for Integers (Signed).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Integer",
    label = "not an Int"
)]
pub trait Int {}

/// Type-level Addition: `Lhs + Rhs`
#[diagnostic::on_unimplemented(
    message = "Cannot add `{Rhs}` to `{Self}`",
    label = "addition not implemented",
    note = "ensure `{Self}` implements `TAdd<{Rhs}>`"
)]
pub trait TAdd<Rhs> {
    type Output;
}

/// Type-level Subtraction: `Lhs - Rhs`
pub trait TSub<Rhs> {
    type Output;
}

/// Type-level Multiplication: `Lhs * Rhs`
pub trait TMul<Rhs> {
    type Output;
}

/// Type-level Division: `Lhs / Rhs`
pub trait TDiv<Rhs> {
    type Output;
}

/// Type-level Remainder: `Lhs % Rhs`
pub trait TRem<Rhs> {
    type Output;
}

/// Type-level Exponentiation: `Lhs ^ Rhs`
pub trait TPow<Rhs> {
    type Output;
}

/// Type-level Boolean Logic.
///
/// Implementors: `True`, `False`, `B0`, `B1`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level Boolean",
    label = "not a Bool",
    note = "implement `Bool` for `{Self}` to use it in logical operations"
)]
pub trait Bool {
    /// Result of `!Self`
    type Not;
    /// Result of `Self && Rhs`
    type And<Rhs: Bool>;
    /// Result of `Self || Rhs`
    type Or<Rhs: Bool>;
    /// Result of `Self ^ Rhs`
    type Xor<Rhs: Bool>;
    /// Result of `!(Self && Rhs)`
    type Nand<Rhs: Bool>;
    /// Result of `!(Self || Rhs)`
    type Nor<Rhs: Bool>;
    /// Result of `!(Self ^ Rhs)`
    type Xnor<Rhs: Bool>;
}

/// Type-level List (Cons List).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid Type-Level List",
    label = "not a List",
    note = "implement `List` for `{Self}` to use it in list operations like `Map`, `Filter`, `Fold`"
)]
pub trait List {
    /// The type of the head element.
    type Head;
    /// The type of the tail (must also be a List, or Nil).
    type Tail;
    /// Prepend a new element to this list.
    type Cons<NewHead>;
}
