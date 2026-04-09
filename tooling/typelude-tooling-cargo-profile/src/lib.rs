//! Analyze cargo self-profile artifacts and `crox` chrome traces.

use std::{
    collections::{BTreeMap, BinaryHeap},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize, de::DeserializeSeed};
use typelude_tooling_core::{ToolingError, ToolingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoProfileSemanticKind {
    TypeCheck,
    Branching,
    ConstraintSolve,
    ConstEval,
    MirBuild,
    TypeAscription,
    WellFormedness,
    InstanceResolution,
    Layout,
    Monomorphization,
    Linking,
    Unknown,
}

impl CargoProfileSemanticKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TypeCheck => "type_check",
            Self::Branching => "branching",
            Self::ConstraintSolve => "constraint_solve",
            Self::ConstEval => "const_eval",
            Self::MirBuild => "mir_build",
            Self::TypeAscription => "type_ascription",
            Self::WellFormedness => "well_formedness",
            Self::InstanceResolution => "instance_resolution",
            Self::Layout => "layout",
            Self::Monomorphization => "monomorphization",
            Self::Linking => "linking",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoProfileDomainKind {
    VmState,
    RunVm,
    HostRequest,
    PeanoNat,
    TypeList,
    TypeApply,
}

impl CargoProfileDomainKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::VmState => "vm_state",
            Self::RunVm => "run_vm",
            Self::HostRequest => "host_request",
            Self::PeanoNat => "peano_nat",
            Self::TypeList => "type_list",
            Self::TypeApply => "type_apply",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationCount {
    pub label: String,
    pub total_us: u64,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChromeLongestEvent {
    pub name: String,
    pub category: String,
    pub total_us: u64,
    pub thread_id: Option<u64>,
    pub start_us: Option<u64>,
    pub arg0: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryHotspot {
    pub name: String,
    pub total_us: u64,
    pub count: u64,
    pub max_us: u64,
    pub example_arg0: Option<String>,
    pub semantic_kinds: Vec<CargoProfileSemanticKind>,
    pub domain_kinds: Vec<CargoProfileDomainKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemHotspot {
    pub item: String,
    pub total_us: u64,
    pub query_breakdown: Vec<DurationCount>,
    pub semantic_breakdown: Vec<DurationCount>,
    pub domain_breakdown: Vec<DurationCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChromeProfileAnalysis {
    pub event_count: u64,
    pub total_duration_us: u64,
    pub category_breakdown: Vec<DurationCount>,
    pub thread_breakdown: Vec<DurationCount>,
    pub query_hotspots: Vec<QueryHotspot>,
    pub semantic_hotspots: Vec<DurationCount>,
    pub domain_hotspots: Vec<DurationCount>,
    pub item_hotspots: Vec<ItemHotspot>,
    pub longest_events: Vec<ChromeLongestEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfProfileQueryStat {
    pub label: String,
    pub total_us: u64,
    pub self_us: u64,
    pub invocation_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSize {
    pub label: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfProfileSummaryAnalysis {
    pub total_time_us: u64,
    pub top_self_time: Vec<SelfProfileQueryStat>,
    pub top_total_time: Vec<SelfProfileQueryStat>,
    pub artifacts: Vec<ArtifactSize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoProfileAnalysis {
    pub chrome: Option<ChromeProfileAnalysis>,
    pub self_profile: Option<SelfProfileSummaryAnalysis>,
    pub findings: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CargoProfileAnalyzer;

impl CargoProfileAnalyzer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn analyze_chrome_profiler_path(
        &self,
        path: impl AsRef<Path>,
        top: usize,
    ) -> ToolingResult<ChromeProfileAnalysis> {
        let file = fs::File::open(path)?;
        self.analyze_chrome_profiler_reader(file, top)
    }

    pub fn analyze_chrome_profiler_reader<R: Read>(
        &self,
        reader: R,
        top: usize,
    ) -> ToolingResult<ChromeProfileAnalysis> {
        let mut state = ChromeAggregate::default();
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        ChromeAggregateSeed {
            state: &mut state,
        }
        .deserialize(&mut deserializer)?;
        Ok(state.finish(top))
    }

    pub fn analyze_summarize_json_path(
        &self,
        path: impl AsRef<Path>,
        top: usize,
    ) -> ToolingResult<SelfProfileSummaryAnalysis> {
        self.analyze_summarize_json_str(&fs::read_to_string(path)?, top)
    }

    pub fn analyze_summarize_json_str(
        &self,
        input: &str,
        top: usize,
    ) -> ToolingResult<SelfProfileSummaryAnalysis> {
        let parsed: SummarizeJson = serde_json::from_str(input)?;
        Ok(parsed.finish(top))
    }

    pub fn analyze_self_profile_prefix(
        &self,
        prefix: impl AsRef<Path>,
        top: usize,
    ) -> ToolingResult<SelfProfileSummaryAnalysis> {
        let json = run_summarize_json(prefix.as_ref())?;
        self.analyze_summarize_json_str(&json, top)
    }

    pub fn analyze_paths(
        &self,
        chrome_profiler_json: Option<&Path>,
        summarize_json: Option<&Path>,
        self_profile_prefix: Option<&Path>,
        top: usize,
    ) -> ToolingResult<CargoProfileAnalysis> {
        let chrome = chrome_profiler_json
            .map(|path| self.analyze_chrome_profiler_path(path, top))
            .transpose()?;
        let self_profile = match (summarize_json, self_profile_prefix) {
            (Some(path), _) => Some(self.analyze_summarize_json_path(path, top)?),
            (None, Some(prefix)) => Some(self.analyze_self_profile_prefix(prefix, top)?),
            (None, None) => None,
        };
        let findings = infer_findings(chrome.as_ref(), self_profile.as_ref());
        Ok(CargoProfileAnalysis {
            chrome,
            self_profile,
            findings,
        })
    }
}

#[must_use]
pub fn render_text(analysis: &CargoProfileAnalysis) -> String {
    let mut lines = Vec::new();

    if !analysis.findings.is_empty() {
        lines.push(String::from("findings:"));
        for finding in &analysis.findings {
            lines.push(format!("- {finding}"));
        }
    }

    if let Some(self_profile) = &analysis.self_profile {
        lines.push(format!("self_profile.total_time_us={}", self_profile.total_time_us));
        if !self_profile.top_self_time.is_empty() {
            lines.push(String::from("self_profile.top_self_time:"));
            for query in &self_profile.top_self_time {
                lines.push(format!(
                    "- {} self_us={} total_us={} invocations={} cache_hits={} cache_misses={}",
                    query.label,
                    query.self_us,
                    query.total_us,
                    query.invocation_count,
                    query.cache_hits,
                    query.cache_misses
                ));
            }
        }
        if !self_profile.artifacts.is_empty() {
            lines.push(String::from("self_profile.artifacts:"));
            for artifact in &self_profile.artifacts {
                lines.push(format!("- {} bytes={}", artifact.label, artifact.bytes));
            }
        }
    }

    if let Some(chrome) = &analysis.chrome {
        lines.push(format!("chrome.event_count={}", chrome.event_count));
        lines.push(format!("chrome.total_duration_us={}", chrome.total_duration_us));
        if !chrome.category_breakdown.is_empty() {
            lines.push(String::from("chrome.categories:"));
            for entry in &chrome.category_breakdown {
                lines.push(format!(
                    "- {} total_us={} count={}",
                    entry.label, entry.total_us, entry.count
                ));
            }
        }
        if !chrome.semantic_hotspots.is_empty() {
            lines.push(String::from("chrome.semantic_hotspots:"));
            for entry in &chrome.semantic_hotspots {
                lines.push(format!(
                    "- {} total_us={} count={}",
                    entry.label, entry.total_us, entry.count
                ));
            }
        }
        if !chrome.domain_hotspots.is_empty() {
            lines.push(String::from("chrome.domain_hotspots:"));
            for entry in &chrome.domain_hotspots {
                lines.push(format!(
                    "- {} total_us={} count={}",
                    entry.label, entry.total_us, entry.count
                ));
            }
        }
        if !chrome.query_hotspots.is_empty() {
            lines.push(String::from("chrome.query_hotspots:"));
            for entry in &chrome.query_hotspots {
                let semantics = entry
                    .semantic_kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(",");
                let domains = entry
                    .domain_kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "- {} total_us={} count={} max_us={} semantics=[{}] domains=[{}] arg0={}",
                    entry.name,
                    entry.total_us,
                    entry.count,
                    entry.max_us,
                    semantics,
                    domains,
                    entry.example_arg0.as_deref().unwrap_or("<none>")
                ));
            }
        }
        if !chrome.item_hotspots.is_empty() {
            lines.push(String::from("chrome.item_hotspots:"));
            for entry in &chrome.item_hotspots {
                let dominant_query =
                    entry.query_breakdown.first().map_or("<none>", |value| value.label.as_str());
                let dominant_semantic = entry
                    .semantic_breakdown
                    .first()
                    .map_or("<none>", |value| value.label.as_str());
                let dominant_domain =
                    entry.domain_breakdown.first().map_or("<none>", |value| value.label.as_str());
                lines.push(format!(
                    "- {} total_us={} dominant_query={} dominant_semantic={} dominant_domain={}",
                    entry.item, entry.total_us, dominant_query, dominant_semantic, dominant_domain
                ));
            }
        }
    }

    lines.join("\n")
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ChromeArgs {
    #[serde(default)]
    arg0: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ChromeEvent {
    name: String,
    #[serde(default)]
    cat: Option<String>,
    #[serde(default, rename = "ph")]
    _phase: Option<String>,
    #[serde(default)]
    ts: Option<u64>,
    #[serde(default)]
    dur: Option<u64>,
    #[serde(default)]
    tid: Option<u64>,
    #[serde(default)]
    args: Option<ChromeArgs>,
}

#[derive(Debug, Clone, Default)]
struct QueryAggregate {
    total_us: u64,
    count: u64,
    max_us: u64,
    example_arg0: Option<String>,
    semantics: Vec<CargoProfileSemanticKind>,
    domains: Vec<CargoProfileDomainKind>,
}

#[derive(Debug, Clone, Default)]
struct ItemAggregate {
    total_us: u64,
    queries: BTreeMap<String, CountState>,
    semantics: BTreeMap<String, CountState>,
    domains: BTreeMap<String, CountState>,
}

#[derive(Debug, Clone, Copy, Default)]
struct CountState {
    total_us: u64,
    count: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct HeapEvent(ChromeLongestEvent);

impl Ord for HeapEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_us.cmp(&other.0.total_us)
    }
}

impl PartialOrd for HeapEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default)]
struct ChromeAggregate {
    event_count: u64,
    total_duration_us: u64,
    categories: BTreeMap<String, CountState>,
    threads: BTreeMap<String, CountState>,
    queries: BTreeMap<String, QueryAggregate>,
    semantics: BTreeMap<String, CountState>,
    domains: BTreeMap<String, CountState>,
    items: BTreeMap<String, ItemAggregate>,
    longest_events: BinaryHeap<std::cmp::Reverse<HeapEvent>>,
}

impl ChromeAggregate {
    fn observe(&mut self, event: ChromeEvent) {
        self.event_count += 1;
        let duration = event.dur.unwrap_or(0);
        self.total_duration_us += duration;

        let category = event.cat.unwrap_or_else(|| String::from("<none>"));
        add_count(self.categories.entry(category.clone()).or_default(), duration);
        add_count(
            self.threads
                .entry(event.tid.map_or_else(|| String::from("<none>"), |value| value.to_string()))
                .or_default(),
            duration,
        );

        let arg0 = event.args.and_then(|args| args.arg0);
        let semantics = classify_semantics(&event.name);
        let domains = arg0.as_deref().map(detect_domains).unwrap_or_default();

        if category == "Query" {
            let query = self.queries.entry(event.name.clone()).or_default();
            query.total_us += duration;
            query.count += 1;
            query.max_us = query.max_us.max(duration);
            if query.example_arg0.is_none() {
                query.example_arg0 = arg0.clone();
            }
            extend_unique(&mut query.semantics, &semantics);
            extend_unique(&mut query.domains, &domains);
        }

        for semantic in &semantics {
            add_count(self.semantics.entry(String::from(semantic.label())).or_default(), duration);
        }
        for domain in &domains {
            add_count(self.domains.entry(String::from(domain.label())).or_default(), duration);
        }

        if let Some(item) = arg0.as_deref().and_then(extract_item_label) {
            let aggregate = self.items.entry(item).or_default();
            aggregate.total_us += duration;
            add_count(aggregate.queries.entry(event.name.clone()).or_default(), duration);
            for semantic in &semantics {
                add_count(
                    aggregate.semantics.entry(String::from(semantic.label())).or_default(),
                    duration,
                );
            }
            for domain in &domains {
                add_count(
                    aggregate.domains.entry(String::from(domain.label())).or_default(),
                    duration,
                );
            }
        }

        push_longest(
            &mut self.longest_events,
            ChromeLongestEvent {
                name: event.name,
                category,
                total_us: duration,
                thread_id: event.tid,
                start_us: event.ts,
                arg0,
            },
            20,
        );
    }

    fn finish(self, top: usize) -> ChromeProfileAnalysis {
        ChromeProfileAnalysis {
            event_count: self.event_count,
            total_duration_us: self.total_duration_us,
            category_breakdown: top_count_map(self.categories, top),
            thread_breakdown: top_count_map(self.threads, top),
            query_hotspots: top_queries(self.queries, top),
            semantic_hotspots: top_count_map(self.semantics, top),
            domain_hotspots: top_count_map(self.domains, top),
            item_hotspots: top_items(self.items, top),
            longest_events: self
                .longest_events
                .into_sorted_vec()
                .into_iter()
                .rev()
                .map(|entry| entry.0.0)
                .take(top)
                .collect(),
        }
    }
}

struct ChromeAggregateSeed<'a> {
    state: &'a mut ChromeAggregate,
}

impl<'de> DeserializeSeed<'de> for ChromeAggregateSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChromeAggregateVisitor {
            state: self.state,
        })
    }
}

struct ChromeAggregateVisitor<'a> {
    state: &'a mut ChromeAggregate,
}

impl<'de> serde::de::Visitor<'de> for ChromeAggregateVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a chrome profiler event array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        while let Some(event) = seq.next_element::<ChromeEvent>()? {
            self.state.observe(event);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DurationValue {
    secs: u64,
    nanos: u32,
}

impl DurationValue {
    #[must_use]
    fn total_us(&self) -> u64 {
        self.secs.saturating_mul(1_000_000) + u64::from(self.nanos) / 1_000
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SummarizeQueryJson {
    label: String,
    time: DurationValue,
    self_time: DurationValue,
    number_of_cache_misses: u64,
    number_of_cache_hits: u64,
    invocation_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactSizeJson {
    label: String,
    value: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SummarizeJson {
    query_data: Vec<SummarizeQueryJson>,
    artifact_sizes: Vec<ArtifactSizeJson>,
    total_time: DurationValue,
}

impl SummarizeJson {
    fn finish(self, top: usize) -> SelfProfileSummaryAnalysis {
        let mut top_self_time = self.query_data.iter().map(to_query_stat).collect::<Vec<_>>();
        top_self_time.sort_by(|left, right| {
            right
                .self_us
                .cmp(&left.self_us)
                .then_with(|| right.total_us.cmp(&left.total_us))
                .then_with(|| left.label.cmp(&right.label))
        });
        top_self_time.truncate(top);

        let mut top_total_time = self.query_data.iter().map(to_query_stat).collect::<Vec<_>>();
        top_total_time.sort_by(|left, right| {
            right
                .total_us
                .cmp(&left.total_us)
                .then_with(|| right.self_us.cmp(&left.self_us))
                .then_with(|| left.label.cmp(&right.label))
        });
        top_total_time.truncate(top);

        let mut artifacts = self
            .artifact_sizes
            .into_iter()
            .map(|artifact| ArtifactSize {
                label: artifact.label,
                bytes: artifact.value,
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| {
            right.bytes.cmp(&left.bytes).then_with(|| left.label.cmp(&right.label))
        });

        SelfProfileSummaryAnalysis {
            total_time_us: self.total_time.total_us(),
            top_self_time,
            top_total_time,
            artifacts,
        }
    }
}

fn to_query_stat(value: &SummarizeQueryJson) -> SelfProfileQueryStat {
    SelfProfileQueryStat {
        label: value.label.clone(),
        total_us: value.time.total_us(),
        self_us: value.self_time.total_us(),
        invocation_count: value.invocation_count,
        cache_hits: value.number_of_cache_hits,
        cache_misses: value.number_of_cache_misses,
    }
}

fn add_count(state: &mut CountState, duration: u64) {
    state.total_us += duration;
    state.count += 1;
}

fn top_count_map(input: BTreeMap<String, CountState>, top: usize) -> Vec<DurationCount> {
    let mut rows = input
        .into_iter()
        .map(|(label, state)| DurationCount {
            label,
            total_us: state.total_us,
            count: state.count,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .total_us
            .cmp(&left.total_us)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.label.cmp(&right.label))
    });
    rows.truncate(top);
    rows
}

fn top_queries(input: BTreeMap<String, QueryAggregate>, top: usize) -> Vec<QueryHotspot> {
    let mut rows = input
        .into_iter()
        .map(|(name, aggregate)| QueryHotspot {
            name,
            total_us: aggregate.total_us,
            count: aggregate.count,
            max_us: aggregate.max_us,
            example_arg0: aggregate.example_arg0.map(|value| compact_text(&value, 160)),
            semantic_kinds: aggregate.semantics,
            domain_kinds: aggregate.domains,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .total_us
            .cmp(&left.total_us)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.name.cmp(&right.name))
    });
    rows.truncate(top);
    rows
}

fn top_items(input: BTreeMap<String, ItemAggregate>, top: usize) -> Vec<ItemHotspot> {
    let mut rows = input
        .into_iter()
        .map(|(item, aggregate)| ItemHotspot {
            item,
            total_us: aggregate.total_us,
            query_breakdown: top_count_map(aggregate.queries, 5),
            semantic_breakdown: top_count_map(aggregate.semantics, 5),
            domain_breakdown: top_count_map(aggregate.domains, 5),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right.total_us.cmp(&left.total_us).then_with(|| left.item.cmp(&right.item))
    });
    rows.truncate(top);
    rows
}

fn classify_semantics(query: &str) -> Vec<CargoProfileSemanticKind> {
    let kind = match query {
        "typeck" | "analysis" => CargoProfileSemanticKind::TypeCheck,
        "check_match" => CargoProfileSemanticKind::Branching,
        "evaluate_obligation" => CargoProfileSemanticKind::ConstraintSolve,
        "eval_to_const_value_raw" | "eval_to_allocation_raw" | "trivial_const" => {
            CargoProfileSemanticKind::ConstEval
        },
        "mir_built"
        | "mir_for_ctfe"
        | "mir_drops_elaborated_and_const_checked"
        | "mir_borrowck"
        | "mir_promoted" => CargoProfileSemanticKind::MirBuild,
        "type_op_ascribe_user_type" => CargoProfileSemanticKind::TypeAscription,
        "check_type_wf" | "check_well_formed" | "predicates_of" => {
            CargoProfileSemanticKind::WellFormedness
        },
        "resolve_instance_raw" => CargoProfileSemanticKind::InstanceResolution,
        "layout_of" => CargoProfileSemanticKind::Layout,
        "collect_and_partition_mono_items"
        | "monomorphization_collector"
        | "monomorphization_collector_graph_walk" => CargoProfileSemanticKind::Monomorphization,
        "link" | "link_crate" | "link_binary" | "run_linker" => CargoProfileSemanticKind::Linking,
        _ => CargoProfileSemanticKind::Unknown,
    };
    vec![kind]
}

fn detect_domains(arg0: &str) -> Vec<CargoProfileDomainKind> {
    let mut domains = Vec::new();
    if arg0.contains("VmState<") {
        domains.push(CargoProfileDomainKind::VmState);
    }
    if arg0.contains("RunVm<") {
        domains.push(CargoProfileDomainKind::RunVm);
    }
    if arg0.contains("HostRequest<") {
        domains.push(CargoProfileDomainKind::HostRequest);
    }
    if arg0.contains("peano::Succ<") || arg0.contains("UInt<") || arg0.contains("UTerm") {
        domains.push(CargoProfileDomainKind::PeanoNat);
    }
    if arg0.contains("TArr<") || arg0.contains("Array<") || arg0.contains("TTerm") {
        domains.push(CargoProfileDomainKind::TypeList);
    }
    if arg0.contains("Apply<") || arg0.contains("EApp<") {
        domains.push(CargoProfileDomainKind::TypeApply);
    }
    domains
}

fn extract_item_label(arg0: &str) -> Option<String> {
    find_path_like(arg0, "program_fibonacci::")
        .or_else(|| find_path_like(arg0, "typelude_vm::"))
        .or_else(|| find_path_like(arg0, "typelude_std::"))
        .or_else(|| find_path_like(arg0, "typelude_num::"))
        .or_else(|| find_path_like(arg0, "typelude_col::"))
}

fn find_path_like(input: &str, needle: &str) -> Option<String> {
    let start = input.find(needle)?;
    let suffix = &input[start..];
    let end = suffix
        .char_indices()
        .find_map(|(index, ch)| {
            if index == 0 {
                None
            } else if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_') {
                None
            } else {
                Some(index)
            }
        })
        .unwrap_or(suffix.len());
    Some(suffix[..end].to_string())
}

fn compact_text(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        input.to_string()
    } else {
        format!("{}...", &input[..limit.saturating_sub(3)])
    }
}

fn extend_unique<T: PartialEq + Copy>(into: &mut Vec<T>, values: &[T]) {
    for value in values {
        if !into.contains(value) {
            into.push(*value);
        }
    }
}

fn push_longest(
    heap: &mut BinaryHeap<std::cmp::Reverse<HeapEvent>>,
    event: ChromeLongestEvent,
    keep: usize,
) {
    heap.push(std::cmp::Reverse(HeapEvent(event)));
    while heap.len() > keep {
        let _ = heap.pop();
    }
}

fn infer_findings(
    chrome: Option<&ChromeProfileAnalysis>,
    self_profile: Option<&SelfProfileSummaryAnalysis>,
) -> Vec<String> {
    let mut findings = Vec::new();

    if let Some(summary) = self_profile {
        if let Some(query) = summary.top_self_time.first() {
            findings.push(format!(
                "self-profile self time is dominated by `{}` ({} ms self)",
                query.label,
                query.self_us / 1_000
            ));
        }
        if let Some(query) =
            summary.top_self_time.iter().find(|query| query.label == "evaluate_obligation")
        {
            findings.push(format!(
                "`evaluate_obligation` has {} invocations with {} cache hits and {} cache misses",
                query.invocation_count, query.cache_hits, query.cache_misses
            ));
        }
    }

    if let Some(chrome) = chrome {
        if let Some(item) = chrome.item_hotspots.first() {
            findings.push(format!(
                "chrome trace hotspot is `{}` ({} ms inclusive)",
                item.item,
                item.total_us / 1_000
            ));
        }
        if let Some(semantic) = chrome.semantic_hotspots.first() {
            findings.push(format!(
                "dominant reconstructed semantic signal is `{}` ({} ms)",
                semantic.label,
                semantic.total_us / 1_000
            ));
        }
        if let Some(domain) = chrome.domain_hotspots.first() {
            findings.push(format!(
                "dominant domain-shaped payload is `{}` ({} ms)",
                domain.label,
                domain.total_us / 1_000
            ));
        }
    }

    findings
}

fn normalize_profile_prefix(path: &Path) -> PathBuf {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return path.to_path_buf();
    };
    if let Some(stripped) = file_name.strip_suffix(".mm_profdata") {
        path.with_file_name(stripped)
    } else {
        path.to_path_buf()
    }
}

fn run_summarize_json(prefix: &Path) -> ToolingResult<String> {
    let prefix = normalize_profile_prefix(prefix);
    let source = prefix.with_extension("mm_profdata");
    if !source.exists() {
        return Err(ToolingError::Parse(format!(
            "self-profile artifact not found: {}",
            source.display()
        )));
    }

    let temp_root = std::env::temp_dir().join(format!(
        "typelude-tooling-cargo-profile-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |value| value.as_nanos())
    ));
    fs::create_dir_all(&temp_root)?;

    let staged_profile = temp_root.join(
        source
            .file_name()
            .ok_or_else(|| ToolingError::Parse(String::from("invalid self-profile file name")))?,
    );
    if fs::hard_link(&source, &staged_profile).is_err() {
        fs::copy(&source, &staged_profile)?;
    }

    let staged_prefix = normalize_profile_prefix(&staged_profile);
    let output =
        Command::new("summarize").arg("summarize").arg("--json").arg(&staged_prefix).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&temp_root);
        return Err(ToolingError::Command(format!(
            "`summarize summarize --json` failed: {stderr}"
        )));
    }

    let json = fs::read_to_string(staged_prefix.with_extension("json"))?;
    let _ = fs::remove_dir_all(&temp_root);
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::{CargoProfileAnalyzer, CargoProfileDomainKind, CargoProfileSemanticKind};

    const SAMPLE_CHROME: &str = r#"
[
  {"name":"typeck","cat":"Query","ph":"X","ts":1,"dur":100,"tid":3,"args":{"arg0":"program_fibonacci::demo[1]::_"}},
  {"name":"check_match","cat":"Query","ph":"X","ts":2,"dur":80,"tid":3,"args":{"arg0":"program_fibonacci::demo[1]::_"}},
  {"name":"evaluate_obligation","cat":"Query","ph":"X","ts":3,"dur":60,"tid":3,"args":{"arg0":"CanonicalQueryInput { value: <typelude_vm::vm::runtime::run::RunVm<typelude_vm::vm::semantics::state::VmState<typelude_col::TArr<typelude_num::peano::Succ<typelude_num::peano::Zero>>>> as Trait> }"}},
  {"name":"type_op_ascribe_user_type","cat":"Query","ph":"X","ts":4,"dur":40,"tid":3,"args":{"arg0":"typelude_num::peano::Succ<typelude_num::peano::Zero>"}},
  {"name":"parse_crate","cat":"GenericActivity","ph":"X","ts":5,"dur":10,"tid":3}
]
"#;

    const SAMPLE_SUMMARIZE: &str = r#"
{
  "query_data": [
    {
      "label": "typeck",
      "time": { "secs": 2, "nanos": 0 },
      "self_time": { "secs": 1, "nanos": 500000000 },
      "number_of_cache_misses": 5,
      "number_of_cache_hits": 10,
      "invocation_count": 3,
      "blocked_time": { "secs": 0, "nanos": 0 },
      "incremental_load_time": { "secs": 0, "nanos": 0 },
      "incremental_hashing_time": { "secs": 0, "nanos": 0 }
    },
    {
      "label": "evaluate_obligation",
      "time": { "secs": 0, "nanos": 400000000 },
      "self_time": { "secs": 0, "nanos": 100000000 },
      "number_of_cache_misses": 7,
      "number_of_cache_hits": 99,
      "invocation_count": 42,
      "blocked_time": { "secs": 0, "nanos": 0 },
      "incremental_load_time": { "secs": 0, "nanos": 0 },
      "incremental_hashing_time": { "secs": 0, "nanos": 0 }
    }
  ],
  "artifact_sizes": [
    { "label": "linked_artifact", "value": 1234 }
  ],
  "total_time": { "secs": 3, "nanos": 0 }
}
"#;

    #[test]
    fn analyzes_chrome_profile_stream() {
        let analysis = CargoProfileAnalyzer::new()
            .analyze_chrome_profiler_reader(SAMPLE_CHROME.as_bytes(), 10)
            .expect("chrome profile should parse");

        assert_eq!(analysis.event_count, 5);
        assert_eq!(analysis.query_hotspots[0].name, "typeck");
        assert!(
            analysis
                .semantic_hotspots
                .iter()
                .any(|entry| entry.label == CargoProfileSemanticKind::TypeCheck.label())
        );
        assert!(
            analysis
                .domain_hotspots
                .iter()
                .any(|entry| entry.label == CargoProfileDomainKind::VmState.label())
        );
        assert_eq!(analysis.item_hotspots[0].item, "program_fibonacci::demo");
    }

    #[test]
    fn analyzes_summarize_json() {
        let analysis = CargoProfileAnalyzer::new()
            .analyze_summarize_json_str(SAMPLE_SUMMARIZE, 10)
            .expect("summarize json should parse");

        assert_eq!(analysis.total_time_us, 3_000_000);
        assert_eq!(analysis.top_self_time[0].label, "typeck");
        assert_eq!(analysis.top_total_time[1].label, "evaluate_obligation");
        assert_eq!(analysis.artifacts[0].label, "linked_artifact");
    }

    #[test]
    fn merges_findings_from_both_sources() {
        let analyzer = CargoProfileAnalyzer::new();
        let chrome = analyzer
            .analyze_chrome_profiler_reader(SAMPLE_CHROME.as_bytes(), 10)
            .expect("chrome profile should parse");
        let self_profile = analyzer
            .analyze_summarize_json_str(SAMPLE_SUMMARIZE, 10)
            .expect("summarize json should parse");
        let merged = super::CargoProfileAnalysis {
            findings: super::infer_findings(Some(&chrome), Some(&self_profile)),
            chrome: Some(chrome),
            self_profile: Some(self_profile),
        };
        assert!(merged.chrome.is_some());
        assert!(merged.self_profile.is_some());
        assert!(!merged.findings.is_empty());
    }

    #[test]
    fn empty_analysis_has_no_findings() {
        let merged = CargoProfileAnalyzer::new()
            .analyze_paths(None, None, None, 10)
            .expect("empty analysis should succeed");
        assert!(merged.findings.is_empty());
    }
}
