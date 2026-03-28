//! Typelude-specific semantic adapters and renderers.

pub mod analysis;
pub mod caps;
pub mod diagnostics;
pub mod graph;
pub mod mapper;
pub mod metrics;
pub mod naming;
pub mod render;
pub mod type_expr;

pub use analysis::{GraphAnalysis, GraphSummary, KindDistribution};
pub use caps::{CapabilityKind, normalize_capability_name};
pub use diagnostics::TypeludeDiagnosticEnricher;
pub use graph::TraceGraphBuilder;
pub use mapper::{SemanticMapper, SemanticNode, SemanticNodeKind};
pub use metrics::TypeludeMetricEnricher;
pub use naming::compress_symbol_name;
pub use render::TypeludeRenderer;
pub use type_expr::{SemanticExpr, TypeExpr};
