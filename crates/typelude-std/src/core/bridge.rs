//! Reflection and reification traits for the core type-level model.

/// Type to runtime value reification.
pub trait Reify<T> {
    const REIFIED: T;

    fn reify() -> T {
        Self::REIFIED
    }
}

/// Compatibility marker for APIs that want an explicit "value-side" boundary.
pub trait IntoValue<T>: Reify<T> {}

impl<T, A> IntoValue<T> for A where A: Reify<T> {}

/// Runtime/tag-side representation to type-level representation conversion.
pub trait Reflect<Target> {
    type Output;
}

/// Common interface for lifting compile-time values into type-level values.
pub trait Lift {
    type Output;
}
