use crate::vm::{run::direct::Machine, state::VmState};

pub trait GetStack {
    type Output;
}

pub trait GetLocals {
    type Output;
}

pub trait GetMemory {
    type Output;
}

pub trait GetFrames {
    type Output;
}

pub trait GetLabels {
    type Output;
}

pub trait GetProgram {
    type Output;
}

pub trait SetProgram<NewProgram> {
    type Output;
}

impl<S, L, M, F, B, P> GetStack for VmState<S, L, M, F, B, P> {
    type Output = S;
}

impl<S, L, M, F, B, P> GetLocals for VmState<S, L, M, F, B, P> {
    type Output = L;
}

impl<S, L, M, F, B, P> GetMemory for VmState<S, L, M, F, B, P> {
    type Output = M;
}

impl<S, L, M, F, B, P> GetFrames for VmState<S, L, M, F, B, P> {
    type Output = F;
}

impl<S, L, M, F, B, P> GetLabels for VmState<S, L, M, F, B, P> {
    type Output = B;
}

impl<S, L, M, F, B, P> GetProgram for VmState<S, L, M, F, B, P> {
    type Output = P;
}

impl<S, L, M, F, B, P, NewProgram> SetProgram<NewProgram> for VmState<S, L, M, F, B, P> {
    type Output = VmState<S, L, M, F, B, NewProgram>;
}

impl<S, L, M, F, B, P, Meta, Fx> GetStack for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = S;
}

impl<S, L, M, F, B, P, Meta, Fx> GetLocals for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = L;
}

impl<S, L, M, F, B, P, Meta, Fx> GetMemory for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = M;
}

impl<S, L, M, F, B, P, Meta, Fx> GetFrames for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = F;
}

impl<S, L, M, F, B, P, Meta, Fx> GetLabels for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = B;
}

impl<S, L, M, F, B, P, Meta, Fx> GetProgram for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = P;
}
