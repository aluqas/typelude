#![recursion_limit = "4096"]

#[path = "integration/support.rs"]
mod support;

#[path = "integration/parity_control.rs"]
mod parity_control;

#[path = "integration/parity_effects.rs"]
mod parity_effects;

#[path = "integration/program_fibonacci.rs"]
mod program_fibonacci;

#[path = "integration/program_memory_copy.rs"]
mod program_memory_copy;

#[path = "integration/program_nested_calls.rs"]
mod program_nested_calls;

#[path = "integration/scenario_trace_semantics.rs"]
mod scenario_trace_semantics;

#[path = "integration/scenario_trap_paths.rs"]
mod scenario_trap_paths;
