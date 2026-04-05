#![feature(rustc_private)]
#![allow(unused_extern_crates)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

pub(crate) mod adapters;
pub mod emit;
pub mod error;
pub mod filters;
pub mod frontends;
pub mod hooks;
pub mod queries;
pub mod session;
pub mod subjects;

use emit::emit_raw;
use frontends::{CollectConfig, run_collect_frontend, run_owner_query_frontend};
use rustc_driver::Callbacks;
use rustc_interface::Config;
use typelude_tooling_core::{ErrorRaised, GoalResult, OwnerQuerySpec, TraceEvent, TracePayload};

struct TypeludeCallbacks {
    collect_config: CollectConfig,
}

impl TypeludeCallbacks {
    fn new() -> Self {
        Self {
            collect_config: CollectConfig::default(),
        }
    }
}

impl Callbacks for TypeludeCallbacks {
    fn config(&mut self, config: &mut Config) {
        let _ = config;
        // TODO: enable next_solver once derive ICEs are fixed upstream.
    }

    fn after_analysis(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        let result = if let Some(query) = OwnerQuerySpec::from_env() {
            run_owner_query_frontend(
                tcx,
                &self.collect_config,
                &query.owner,
                query.target_kind,
                query.match_kind,
            )
        } else {
            run_collect_frontend(tcx, &self.collect_config)
        };
        if let Err(error) = result {
            let event = TraceEvent::new(
                typelude_tooling_core::EventId::new(0),
                TracePayload::ErrorRaised(ErrorRaised {
                    hook_id: None,
                    subject_id: None,
                    goal_id: None,
                    result: GoalResult::Error,
                    message: error.to_string(),
                }),
            );
            emit_raw(&event);
        }
        rustc_driver::Compilation::Continue
    }
}

pub fn main_entry() -> std::process::ExitCode {
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && !args[1].starts_with('-') {
        args.remove(1);
    }

    rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&args, &mut TypeludeCallbacks::new());
    })
}
