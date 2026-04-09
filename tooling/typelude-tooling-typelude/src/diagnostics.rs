use typelude_tooling_core::{CapabilityFailure, DiagnosticKind, DiagnosticRecord};

use crate::caps::{CapabilityKind, classify_capability, normalize_capability_name};

#[derive(Debug, Default)]
pub struct TypeludeDiagnosticEnricher;

impl TypeludeDiagnosticEnricher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Enrich a [`DiagnosticRecord`] with typelude-specific context.
    ///
    /// For capability failures (E0277), normalises the trait name and appends
    /// a `typelude capability required:` note.
    #[must_use]
    pub fn enrich(&self, diagnostic: &DiagnosticRecord) -> DiagnosticRecord {
        let mut enriched = diagnostic.clone();

        if let DiagnosticKind::CapabilityFailure(capability) = diagnostic.kind() {
            let normalized = capability
                .required
                .as_deref()
                .map(normalize_capability_name)
                .or_else(|| extract_capability_from_text(&diagnostic.message()));

            enriched.data.normalized = DiagnosticKind::CapabilityFailure(CapabilityFailure {
                required: normalized.clone(),
                actual: capability.actual.clone(),
                context: capability.context.clone().or_else(|| Some(diagnostic.message())),
            });

            if let Some(required) = normalized {
                enriched.notes.push(format!("typelude capability required: {required}"));
            }
        }

        enriched
    }

    /// Generate a human-readable explanation of why a typelude type-level
    /// computation failed.
    ///
    /// Returns `None` if the diagnostic is not a capability failure or the
    /// failure kind cannot be mapped to a known typelude construct.
    #[must_use]
    pub fn explain_failure(&self, diagnostic: &DiagnosticRecord) -> Option<String> {
        let DiagnosticKind::CapabilityFailure(capability) = diagnostic.kind() else {
            return None;
        };

        let fallback = extract_from_message(&diagnostic.message());
        let raw =
            capability.required.as_deref().or_else(|| fallback.as_deref()).unwrap_or("unknown");

        let kind = classify_capability(raw);

        // Include the original message as additional context when available.
        let context_hint = capability
            .context
            .as_deref()
            .filter(|c| !c.is_empty())
            .map_or_else(String::new, |c| format!("\n  Context: {c}"));

        let body = match kind {
            CapabilityKind::Bool => format!(
                "`EIf` and `EWhile` require the condition to evaluate to `True` or `False`, \
                 but the type does not implement `IsBool`.\n\
                 \n\
                 Fix: ensure the predicate `TyFn` returns `True` or `False` from typelude_std.\
                 {context_hint}"
            ),

            CapabilityKind::List => format!(
                "This list operation requires a typelude cons-list (`Array<H, T>` or `Nil`), \
                 but the type does not implement `IsList`.\n\
                 \n\
                 Fix: use `tyarray![...]` or `Array<..., Nil>` to construct a list.\
                 {context_hint}"
            ),

            CapabilityKind::Callable => format!(
                "`EApp` (function application) requires the function argument to implement `TyFn`, \
                 but got a non-callable type.\n\
                 \n\
                 Fix: define the function with `ty_fn!` or implement `TyFn` manually.\
                 {context_hint}"
            ),

            CapabilityKind::LookupKey => format!(
                "`EGet` (key lookup) requires the key to be present in the map or array, \
                 but the key was not found.\n\
                 \n\
                 Fix: verify the key type matches exactly — type-level equality is structural.\
                 {context_hint}"
            ),

            CapabilityKind::SupportsAdd => format!(
                "`EAdd` requires both operands to support type-level addition (`TAdd`). \
                 The operand types must be typenum naturals (`U0`, `U1`, …).\n\
                 \n\
                 Fix: ensure both sides are typenum unsigned integers.\
                 {context_hint}"
            ),

            CapabilityKind::SupportsCompare => format!(
                "This operation requires type-level equality comparison (`EqDecide`). \
                 The types being compared must implement `IsEqual` from typenum.\n\
                 \n\
                 Fix: ensure both types are typenum-comparable (e.g. `U3 == U3`).\
                 {context_hint}"
            ),

            CapabilityKind::Unknown => format!(
                "Trait `{raw}` is not implemented for the given type.\n\
                 \n\
                 This may indicate a missing capability bound, an incorrect type argument, \
                 or a helper dispatch that could not resolve.\
                 {context_hint}"
            ),
        };

        Some(body)
    }
}

fn extract_capability_from_text(input: &str) -> Option<String> {
    let (_, suffix) = input.split_once('`')?;
    let (symbol, _) = suffix.split_once('`')?;
    Some(normalize_capability_name(symbol))
}

/// Like `extract_capability_from_text` but returns the raw (non-normalised)
/// symbol for use in classify_capability.
fn extract_from_message(input: &str) -> Option<String> {
    let (_, suffix) = input.split_once('`')?;
    let (symbol, _) = suffix.split_once('`')?;
    Some(symbol.to_owned())
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::{
        CapabilityFailure, DiagnosticKind, DiagnosticLevel, DiagnosticRecord,
    };

    use super::TypeludeDiagnosticEnricher;

    fn make_capability_record(required: &str, message: &str) -> DiagnosticRecord {
        DiagnosticRecord {
            data: typelude_tooling_core::DiagnosticData {
                raw: typelude_tooling_core::RawCompilerDiagnostic {
                    code: Some(String::from("E0277")),
                    message: message.to_owned(),
                    rendered: None,
                },
                normalized: DiagnosticKind::CapabilityFailure(CapabilityFailure {
                    required: Some(required.to_owned()),
                    actual: None,
                    context: None,
                }),
            },
            level: DiagnosticLevel::Error,
            primary_span: None,
            related_spans: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            span_ids: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn enriches_capability_diagnostics() {
        let record = make_capability_record("Boolish", "expected a typelude Boolish value");
        let enriched = TypeludeDiagnosticEnricher::new().enrich(&record);
        assert!(
            enriched.notes.iter().any(|note| note.contains("typelude capability required: Bool"))
        );
    }

    // ⑩ explain_failure
    #[test]
    fn explains_bool_failure() {
        let record = make_capability_record("IsBool", "IsBool not satisfied");
        let explanation = TypeludeDiagnosticEnricher::new().explain_failure(&record);
        assert!(explanation.is_some());
        let text = explanation.unwrap();
        assert!(text.contains("EIf") && text.contains("True") && text.contains("False"));
    }

    #[test]
    fn explains_callable_failure() {
        let record = make_capability_record("TyFn", "TyFn not satisfied");
        let explanation = TypeludeDiagnosticEnricher::new().explain_failure(&record);
        assert!(explanation.unwrap().contains("EApp"));
    }

    #[test]
    fn explains_list_failure() {
        let record = make_capability_record("IsList", "IsList not satisfied");
        let explanation = TypeludeDiagnosticEnricher::new().explain_failure(&record);
        assert!(explanation.unwrap().contains("IsList"));
    }

    #[test]
    fn explains_unknown_capability() {
        let record = make_capability_record("SomeTrait", "SomeTrait not satisfied");
        let explanation = TypeludeDiagnosticEnricher::new().explain_failure(&record);
        assert!(explanation.unwrap().contains("SomeTrait"));
    }

    #[test]
    fn returns_none_for_non_capability_diagnostic() {
        let record = DiagnosticRecord {
            data: typelude_tooling_core::DiagnosticData {
                raw: typelude_tooling_core::RawCompilerDiagnostic {
                    code: Some(String::from("E0308")),
                    message: String::from("mismatched types"),
                    rendered: None,
                },
                normalized: DiagnosticKind::CompilerDiagnostic,
            },
            level: DiagnosticLevel::Error,
            primary_span: None,
            related_spans: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            span_ids: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
        };
        assert!(TypeludeDiagnosticEnricher::new().explain_failure(&record).is_none());
    }
}
