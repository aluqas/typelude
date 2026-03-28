//! Rustc-facing collectors for typelude tooling.

pub mod diagnostics;
pub mod mir;
pub mod passes;
pub mod profile;
pub mod types;

pub use diagnostics::{RustcDiagnosticsCollector, RustcDiagnosticsConfig};
pub use mir::{MirArtifactCollector, MirArtifactConfig, MirArtifactReport};
pub use passes::TimePassesCollector;
pub use profile::{SelfProfileCollector, SelfProfileConfig, SelfProfileReport};
pub use types::TypeSizesCollector;
