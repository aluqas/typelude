#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    Bool,
    List,
    Callable,
    LookupKey,
    SupportsAdd,
    SupportsCompare,
    Unknown,
}

#[must_use]
pub fn normalize_capability_name(input: &str) -> String {
    let compact = input.trim().trim_matches('`');
    match classify_capability(compact) {
        CapabilityKind::Bool => String::from("Bool"),
        CapabilityKind::List => String::from("List"),
        CapabilityKind::Callable => String::from("Callable"),
        CapabilityKind::LookupKey => String::from("LookupKey"),
        CapabilityKind::SupportsAdd => String::from("SupportsAdd"),
        CapabilityKind::SupportsCompare => String::from("SupportsCompare"),
        CapabilityKind::Unknown => compact.to_owned(),
    }
}

#[must_use]
pub fn classify_capability(input: &str) -> CapabilityKind {
    if input.contains("Bool") {
        CapabilityKind::Bool
    } else if input.contains("List") || input.contains("Array") {
        CapabilityKind::List
    } else if input.contains("Callable") || input.contains("TyFn") {
        CapabilityKind::Callable
    } else if input.contains("LookupKey") || input.contains("Get") {
        CapabilityKind::LookupKey
    } else if input.contains("Add") {
        CapabilityKind::SupportsAdd
    } else if input.contains("Cmp") || input.contains("Compare") || input.contains("Eq") {
        CapabilityKind::SupportsCompare
    } else {
        CapabilityKind::Unknown
    }
}
