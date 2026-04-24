//! Host request protocol types.

use core::marker::PhantomData;

/// Associates a host call signature with the value type pushed back onto the VM
/// stack on resume.
pub trait HostSignature {
    type Response;
}

/// A suspended host request carrying the host signature, the current argument
/// stack, and the response contract required to resume execution.
#[derive(Debug)]
pub struct HostRequest<Sig, Args, Response>(pub PhantomData<(Sig, Args, Response)>);

/// Extracts the response type expected by a suspended host request.
pub trait HostRequestResponse {
    type Output;
}

impl<Sig, Args, Response> HostRequestResponse for HostRequest<Sig, Args, Response> {
    type Output = Response;
}
