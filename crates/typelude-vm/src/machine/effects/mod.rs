use core::marker::PhantomData;

use typelude_std::core::Eval;

pub mod fuel;
pub mod io;
pub mod trace;
pub mod trap;

pub use fuel::{BeforeStep, IgnoreFuel, MeteredFuel, OutOfFuel};
pub use io::{HostRequest, SuspendIo, SuspendIoPolicy};
pub use trace::{EmitTrace, NoTrace, RecordTrace};
pub use trap::{RaiseTrap, TrapAsResult};

#[derive(Debug)]
pub struct Effects<TracePolicy, FuelPolicy, TrapPolicy, IoPolicy>(
    pub PhantomData<(TracePolicy, FuelPolicy, TrapPolicy, IoPolicy)>,
);

impl<T, F, R, I> Eval for Effects<T, F, R, I> {
    type Output = Self;
}

pub type PureEffects = Effects<NoTrace, IgnoreFuel, TrapAsResult, SuspendIoPolicy>;
pub type TraceEffects = Effects<RecordTrace, IgnoreFuel, TrapAsResult, SuspendIoPolicy>;
