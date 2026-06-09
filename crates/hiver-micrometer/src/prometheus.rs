//! Prometheus text format exporter.
//! Prometheus 文本格式导出器。

use crate::registry::MetricRegistry;

/// Export all metrics from a registry in Prometheus text format.
/// 以 Prometheus 文本格式导出注册表中的所有指标。
pub fn to_prometheus_text(registry: &MetricRegistry) -> String {
    let mut lines = Vec::new();

    for counter in registry.get_counters() {
        let name = sanitize_prometheus_name(counter.id().name.as_str());
        let labels = format_labels(&counter.id().tags);
        lines.push(format!("# TYPE {name} counter"));
        lines.push(format!("{name}{labels} {}", counter.count()));
        lines.push(String::new());
    }

    for gauge in registry.get_gauges() {
        let name = sanitize_prometheus_name(gauge.id().name.as_str());
        let labels = format_labels(&gauge.id().tags);
        lines.push(format!("# TYPE {name} gauge"));
        lines.push(format!("{name}{labels} {}", gauge.value()));
        lines.push(String::new());
    }

    for timer in registry.get_timers() {
        let name = sanitize_prometheus_name(timer.id().name.as_str());
        let labels = format_labels(&timer.id().tags);

        lines.push(format!("# TYPE {name}_count summary"));
        lines.push(format!("{name}_count{labels} {}", timer.count()));

        lines.push(format!("# TYPE {name}_sum summary"));
        lines.push(format!("{name}_sum{labels} {:.9}", timer.total_time().as_secs_f64()));

        let max = timer.max_time().unwrap_or_default();
        lines.push(format!("# TYPE {name}_max gauge"));
        lines.push(format!("{name}_max{labels} {:.9}", max.as_secs_f64()));

        let mean = timer.average_time().unwrap_or_default();
        lines.push(format!("# TYPE {name}_mean gauge"));
        lines.push(format!("{name}_mean{labels} {:.9}", mean.as_secs_f64()));

        lines.push(String::new());
    }

    lines.join("\n")
}

/// Sanitize a metric name for Prometheus compatibility.
fn sanitize_prometheus_name(name: &str) -> String {
    name.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                if c.is_ascii_alphabetic() || c == '_' || c == ':' {
                    c
                } else {
                    '_'
                }
            } else if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Format tags as Prometheus label string.
fn format_labels(tags: &crate::metric::Tags) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let pairs: Vec<String> = tags
        .iter()
        .map(|(k, v)| format!("{k}=\"{}\"", escape_label_value(v)))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

/// Escape special characters in label values.
fn escape_label_value(value: &str) -> String {
    value
        .chars()
        .fold(String::with_capacity(value.len()), |mut s, c| {
            match c {
                '\\' => s.push_str("\\\\"),
                '"' => s.push_str("\\\""),
                '\n' => s.push_str("\\n"),
                c => s.push(c),
            }
            s
        })
}

/// HTTP response for `/metrics` endpoint.
pub struct PrometheusResponse {
    /// Response body.
    pub body: String,
    /// Content-Type header value.
    pub content_type: &'static str,
}

impl PrometheusResponse {
    /// Create a response from a registry.
    pub fn from_registry(registry: &MetricRegistry) -> Self {
        Self {
            body: to_prometheus_text(registry),
            content_type: "text/plain; version=0.0.4; charset=utf-8",
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_prometheus_name("http.requests"), "http_requests");
        assert_eq!(sanitize_prometheus_name("request-count"), "request_count");
        assert_eq!(sanitize_prometheus_name("123invalid"), "_23invalid");
    }

    #[test]
    fn test_format_labels_empty() {
        assert_eq!(format_labels(&crate::metric::Tags::new()), "");
    }

    #[test]
    fn test_escape_label_value() {
        assert_eq!(escape_label_value("has\"quote"), "has\\\"quote");
    }

    #[test]
    fn test_prometheus_counter_output() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("http_requests_total").unwrap();
        counter.increment();
        counter.increment();
        let output = to_prometheus_text(&registry);
        assert!(output.contains("# TYPE http_requests_total counter"));
        assert!(output.contains("http_requests_total 2"));
    }

    #[test]
    fn test_prometheus_gauge_output() {
        let registry = MetricRegistry::new();
        let gauge = registry.gauge("active_connections").unwrap();
        gauge.set(42.5);
        let output = to_prometheus_text(&registry);
        assert!(output.contains("# TYPE active_connections gauge"));
        assert!(output.contains("active_connections 42.5"));
    }

    #[test]
    fn test_prometheus_timer_output() {
        let registry = MetricRegistry::new();
        let timer = registry.timer("request_duration").unwrap();
        timer.record(Duration::from_millis(100));
        let output = to_prometheus_text(&registry);
        assert!(output.contains("request_duration_count 1"));
    }

    #[test]
    fn test_prometheus_response() {
        let registry = MetricRegistry::new();
        registry.counter("test_metric").unwrap().increment();
        let resp = PrometheusResponse::from_registry(&registry);
        assert_eq!(resp.content_type, "text/plain; version=0.0.4; charset=utf-8");
        assert!(resp.body.contains("test_metric 1"));
    }
}
