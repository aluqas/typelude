//! Type-to-type conversion traits.

/// Type-level conversion from `Source` into `Self`.
pub trait From<Source> {
    type Output;
}

/// Type-level conversion of `Self` into `Target`.
pub trait Into<Target> {
    type Output;
}

impl<Source, Target> Into<Target> for Source
where
    Target: From<Source>,
{
    type Output = <Target as From<Source>>::Output;
}

impl<Source, Target> From<Source> for Target
where
    Source: Into<Target>,
{
    type Output = <Source as Into<Target>>::Output;
}
