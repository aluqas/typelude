//! Shared self-based capability traits for primitive type-level values.

/// Unary boolean negation.
pub trait Not {
    type Output;
}

/// Binary boolean conjunction.
pub trait And<Rhs> {
    type Output;
}

/// Binary boolean disjunction.
pub trait Or<Rhs> {
    type Output;
}

/// Binary boolean exclusive-or.
pub trait Xor<Rhs> {
    type Output;
}

/// Binary boolean nand.
pub trait Nand<Rhs> {
    type Output;
}

/// Binary addition.
pub trait Add<Rhs> {
    type Output;
}

/// Binary subtraction.
pub trait Sub<Rhs> {
    type Output;
}

/// Binary multiplication.
pub trait Mul<Rhs> {
    type Output;
}

/// Binary division.
pub trait Div<Rhs> {
    type Output;
}

/// Collection length.
pub trait Len {
    type Output;
}

/// Collection head.
pub trait Head {
    type Output;
}

/// Collection tail.
pub trait Tail {
    type Output;
}

/// Indexed collection access.
pub trait Get<Idx> {
    type Output;
}

/// Indexed collection update.
pub trait Set<Idx, Val> {
    type Output;
}

/// Collection concatenation.
pub trait Concat<Other> {
    type Output;
}

/// Append a single element to the end of a collection.
pub trait Append<Elem> {
    type Output;
}

/// Prepend a single element to the front of a collection.
pub trait Prepend<Elem> {
    type Output;
}
