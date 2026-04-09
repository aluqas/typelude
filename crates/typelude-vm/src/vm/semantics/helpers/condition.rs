//! Condition normalization and branch selection.

use typelude_bool::{False, True};

use crate::vm::value::Lit;

#[derive(Debug)]
pub struct BranchTrue;

#[derive(Debug)]
pub struct BranchFalse;

#[derive(Debug)]
pub struct BranchInvalid;

/// Classifies a condition value into taken, untaken, or invalid branch
/// outcomes.
pub trait DecideBranch {
    type Output;
}

impl DecideBranch for Lit<True> {
    type Output = BranchTrue;
}

impl DecideBranch for Lit<False> {
    type Output = BranchFalse;
}

impl DecideBranch for Lit<typenum::B1> {
    type Output = BranchTrue;
}

impl DecideBranch for Lit<typenum::B0> {
    type Output = BranchFalse;
}

impl DecideBranch for Lit<typenum::UTerm> {
    type Output = BranchInvalid;
}

impl<N, B> DecideBranch for Lit<typenum::UInt<N, B>> {
    type Output = BranchInvalid;
}

impl<U> DecideBranch for Lit<typenum::PInt<U>>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = BranchInvalid;
}

impl<U> DecideBranch for Lit<typenum::NInt<U>>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = BranchInvalid;
}

impl DecideBranch for Lit<typenum::Z0> {
    type Output = BranchInvalid;
}
