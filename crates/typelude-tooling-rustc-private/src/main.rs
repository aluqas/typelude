//! typelude rustc driver — captures trait resolution proof trees via
//! `rustc_private`.
//!
//! Usage (as a rustc wrapper):
//!
//! ```sh
//! RUSTC_WRAPPER=typelude-rustc-driver cargo +nightly build -p typelude-vm
//! ```
//!
//! Or directly:
//!
//! ```sh
//! typelude-rustc-driver <rustc args>
//! ```
//!
//! Output goes to stderr as newline-delimited JSON.

#![feature(rustc_private)]
#![allow(unused_extern_crates)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod output;
mod collector;
mod visitor;

use crate::output::TraceEventExt;
use rustc_driver::Callbacks;
use rustc_interface::Config;

struct TypeludeCallbacks {
    collector_config: collector::CollectorConfig,
}

impl TypeludeCallbacks {
    fn new() -> Self {
        Self {
            collector_config: collector::CollectorConfig::default(),
        }
    }
}

impl Callbacks for TypeludeCallbacks {
    fn config(&mut self, config: &mut Config) {
        let _ = config;
        // TODO: enable next_solver once derive ICEs are fixed upstream.
        // config.opts.unstable_opts.next_solver = ...;
    }

    fn after_analysis(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        if let Err(error) = collector::run(tcx, &self.collector_config) {
            let event = typelude_tooling_core::TraceEvent::new(
                typelude_tooling_core::EventId::new(0),
                typelude_tooling_core::TraceEventKind::Info,
                "collector_error",
            );
            let event = event.with_detail(format!("{error}"));
            output::emit_raw(&event);
        }
        rustc_driver::Compilation::Continue
    }
}

fn main() -> std::process::ExitCode {
    // If we are invoked as `RUSTC_WRAPPER`, the first argument is the path to
    // the real `rustc`.  We skip it and forward the rest to our driver.
    let mut args: Vec<String> = std::env::args().collect();

    // When used as RUSTC_WRAPPER, cargo passes the real rustc path as argv[1].
    if args.len() > 1 && !args[1].starts_with('-') {
        args.remove(1);
    }

    rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&args, &mut TypeludeCallbacks::new());
    })
}
