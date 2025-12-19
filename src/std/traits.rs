//! **Core Traits**
//!
//! Core trait definitions for `typelude`.
//!
//! This module defines the "Type Classes" that abstraction layers (like `std`) use.
//! Implementations are provided in their respective modules (`int`, `bool`, `array`).

use crate::eval::{ELit, Eval};

/// Trait representing a function
///
/// Represents a transformation from type to type, used in `EWhile` etc.
pub trait TyFn<Arg> {
    type Output;
}

// Automatically unwrap ELit
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
pub trait TypeNat: Eval {
    // We can add associated types here if needed, e.g., Next, Prev.
}

/// Marker trait for Integers (Signed).
pub trait TypeInt: Eval {
    // Sign, Abs, etc.
}

/// Type-level Addition: `Lhs + Rhs`
pub trait TypeAdd<Rhs> {
    type Output;
}

/// Type-level Subtraction: `Lhs - Rhs`
pub trait TypeSub<Rhs> {
    type Output;
}

/// Type-level Multiplication: `Lhs * Rhs`
pub trait TypeMul<Rhs> {
    type Output;
}

/// Type-level Division: `Lhs / Rhs`
pub trait TypeDiv<Rhs> {
    type Output;
}

/// Type-level Remainder: `Lhs % Rhs`
pub trait TypeRem<Rhs> {
    type Output;
}

/// Type-level Exponentiation: `Lhs ^ Rhs`
pub trait TypePow<Rhs> {
    type Output;
}

// =============================================================================
// Logic Traits (Boolean)
// =============================================================================

/// Type-level Boolean Logic.
///
/// Implementors: `TyTrue`, `TyFalse`, `B0`, `B1`.
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
pub trait TypeList: Eval {
    /// The type of the head element.
    type Head;
    /// The type of the tail (must also be a TypeList, or Nil).
    type Tail;
    /// Prepend a new element to this list.
    type Cons<NewHead>;
}
