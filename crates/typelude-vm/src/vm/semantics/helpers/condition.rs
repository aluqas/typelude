//! Condition normalization and branch selection.

use typelude_std::core::ELit;

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

impl DecideBranch for typelude_std::std::prim::bool::True {
    type Output = BranchTrue;
}

impl DecideBranch for typelude_std::std::prim::bool::False {
    type Output = BranchFalse;
}

impl DecideBranch for typenum::B1 {
    type Output = BranchTrue;
}

impl DecideBranch for typenum::B0 {
    type Output = BranchFalse;
}

impl DecideBranch for typenum::UTerm {
    type Output = BranchInvalid;
}

impl<N, B> DecideBranch for typenum::UInt<N, B> {
    type Output = BranchInvalid;
}

impl<U> DecideBranch for typenum::PInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = BranchInvalid;
}

impl<U> DecideBranch for typenum::NInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = BranchInvalid;
}

impl DecideBranch for typenum::Z0 {
    type Output = BranchInvalid;
}

impl<T> DecideBranch for ELit<T>
where
    T: DecideBranch,
{
    type Output = <T as DecideBranch>::Output;
}
