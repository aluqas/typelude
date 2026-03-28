use typelude_tooling_core::{MetricKind, MetricRecord, ToolingResult};

#[derive(Debug, Default)]
pub struct TimePassesCollector;

impl TimePassesCollector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn collect_from_str(&self, input: &str) -> ToolingResult<Vec<MetricRecord>> {
        let mut metrics = Vec::new();
        for line in input.lines().filter(|line| !line.trim().is_empty()) {
            if let Some(metric) = parse_json_line(line).or_else(|| parse_text_line(line)) {
                metrics.push(metric);
            }
        }
        Ok(metrics)
    }
}

fn parse_json_line(line: &str) -> Option<MetricRecord> {
    if !line.starts_with("time: {") {
        return None;
    }
    let json = line.trim_start_matches("time: ").trim();
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let pass = value.get("pass")?.as_str()?;
    let seconds = value.get("time")?.as_f64()?;

    let mut metric =
        MetricRecord::new(MetricKind::WallTimeMillis, format!("pass:{pass}"), seconds * 1_000.0);
    metric.unit = Some(String::from("ms"));
    Some(metric)
}

fn parse_text_line(line: &str) -> Option<MetricRecord> {
    if !line.starts_with("time:") {
        return None;
    }

    let pass = line.split('\t').next_back()?.trim();
    let prefix = line.split(';').next()?;
    let seconds = prefix.split_whitespace().nth(1)?.parse::<f64>().ok()?;
    let mut metric =
        MetricRecord::new(MetricKind::WallTimeMillis, format!("pass:{pass}"), seconds * 1_000.0);
    metric.unit = Some(String::from("ms"));
    Some(metric)
}

#[cfg(test)]
mod tests {
    use super::TimePassesCollector;

    #[test]
    fn parses_text_time_passes() {
        let collector = TimePassesCollector::new();
        let metrics = collector
            .collect_from_str("time:   0.025; rss:   28MB ->   66MB (  +38MB)\ttotal\n")
            .expect("time-passes should parse");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "pass:total");
    }
}
