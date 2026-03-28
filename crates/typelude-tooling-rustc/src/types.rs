use typelude_tooling_core::{MetricKind, MetricRecord, ToolingResult};

#[derive(Debug, Default)]
pub struct TypeSizesCollector;

impl TypeSizesCollector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn collect_from_str(&self, input: &str) -> ToolingResult<Vec<MetricRecord>> {
        let mut metrics = Vec::new();
        for line in input.lines() {
            if let Some(metric) = parse_line(line) {
                metrics.push(metric);
            }
        }
        Ok(metrics)
    }
}

fn parse_line(line: &str) -> Option<MetricRecord> {
    if !line.starts_with("print-type-size type: `") {
        return None;
    }

    let stripped = line.strip_prefix("print-type-size type: `")?;
    let (type_name, rest) = stripped.split_once("`: ")?;
    let bytes = rest.split_whitespace().next()?.parse::<f64>().ok()?;

    let mut metric = MetricRecord::new(MetricKind::TypeSizeBytes, type_name, bytes);
    metric.unit = Some(String::from("bytes"));
    Some(metric)
}

#[cfg(test)]
mod tests {
    use super::TypeSizesCollector;

    #[test]
    fn parses_type_sizes() {
        let collector = TypeSizesCollector::new();
        let metrics = collector
            .collect_from_str(
                "print-type-size type: `std::option::Option<u32>`: 8 bytes, alignment: 4 bytes\n",
            )
            .expect("type sizes should parse");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "std::option::Option<u32>");
        assert_eq!(metrics[0].value, 8.0);
    }
}
