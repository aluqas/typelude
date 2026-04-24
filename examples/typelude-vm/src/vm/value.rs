//! Canonical VM value wrappers.

use core::marker::PhantomData;

use typelude_std::core::Value;

/// VM value literal carrier.
///
/// VM public programs and states should push raw type-level values through
/// `Lit<T>` so the VM surface stays explicit about what lives on the stack,
/// in locals, and in memory.
#[derive(Debug, Default)]
pub struct Lit<T>(pub PhantomData<T>);

impl<T> Value for Lit<T> {}

/// Extracts the raw type-level value carried by a VM literal.
pub trait Unlit {
    type Output;
}

impl<T> Unlit for Lit<T> {
    type Output = T;
}
