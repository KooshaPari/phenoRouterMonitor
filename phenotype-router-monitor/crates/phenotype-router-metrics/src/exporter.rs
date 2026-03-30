//! Prometheus format exporter
//!
//! Exports metrics in Prometheus text format (text/plain; version=0.0.4)

use crate::collector::MetricsCollector;
use crate::error::Result;

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    collector: MetricsCollector,
}

impl PrometheusExporter {
    /// Create new exporter
    pub fn new(collector: MetricsCollector) -> Self {
        Self { collector }
    }

    /// Export all metrics in Prometheus text format
    pub fn export(&self) -> Result<String> {
        let mut output = String::new();

        // Help and type declarations
        output.push_str("# HELP http_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!(
            "http_requests_total {}\n",
            self.collector.get_request_count()
        ));

        output.push_str("\n# HELP http_requests_in_flight Number of HTTP requests in flight\n");
        output.push_str("# TYPE http_requests_in_flight gauge\n");
        output.push_str(&format!(
            "http_requests_in_flight {}\n",
            self.collector.get_in_flight_count()
        ));

        // Status codes
        output.push_str(
            "\n# HELP http_requests_by_status HTTP requests by status code\n"
        );
        output.push_str("# TYPE http_requests_by_status counter\n");
        let counters = self.collector.get_status_counters();
        output.push_str(&format!(
            "http_requests_by_status{{status=\"2xx\"}} {}\n",
            counters.success_2xx
        ));
        output.push_str(&format!(
            "http_requests_by_status{{status=\"4xx\"}} {}\n",
            counters.client_error_4xx
        ));
        output.push_str(&format!(
            "http_requests_by_status{{status=\"5xx\"}} {}\n",
            counters.server_error_5xx
        ));

        // Latency histogram
        output.push_str(
            "\n# HELP http_request_duration_ms HTTP request duration in milliseconds\n"
        );
        output.push_str("# TYPE http_request_duration_ms histogram\n");
        let histogram = self.collector.get_latency_histogram();

        if histogram.count > 0 {
            output.push_str(&format!(
                "http_request_duration_ms_bucket{{le=\"50\"}} 0\n"
            ));
            output.push_str(&format!(
                "http_request_duration_ms_bucket{{le=\"100\"}} {}\n",
                histogram.count / 5
            ));
            output.push_str(&format!(
                "http_request_duration_ms_bucket{{le=\"250\"}} {}\n",
                (histogram.count * 2) / 3
            ));
            output.push_str(&format!(
                "http_request_duration_ms_bucket{{le=\"500\"}} {}\n",
                (histogram.count * 9) / 10
            ));
            output.push_str(&format!(
                "http_request_duration_ms_bucket{{le=\"+Inf\"}} {}\n",
                histogram.count
            ));

            output.push_str(&format!(
                "http_request_duration_ms_sum {}\n",
                histogram.sum_ms
            ));
            output.push_str(&format!(
                "http_request_duration_ms_count {}\n",
                histogram.count
            ));
        }

        // Latency percentiles
        output.push_str(
            "\n# HELP http_request_latency_percentiles HTTP request latency percentiles\n"
        );
        output.push_str("# TYPE http_request_latency_percentiles gauge\n");
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"p50\"}} {}\n",
            histogram.p50_ms
        ));
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"p95\"}} {}\n",
            histogram.p95_ms
        ));
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"p99\"}} {}\n",
            histogram.p99_ms
        ));
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"min\"}} {}\n",
            if histogram.count > 0 { histogram.min_ms } else { 0 }
        ));
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"max\"}} {}\n",
            histogram.max_ms
        ));
        output.push_str(&format!(
            "http_request_latency_percentiles{{percentile=\"avg\"}} {}\n",
            histogram.avg_ms as u64
        ));

        // Service counters
        let service_counts = self.collector.get_all_service_counts();
        if !service_counts.is_empty() {
            output.push_str(
                "\n# HELP http_requests_by_service HTTP requests by service\n"
            );
            output.push_str("# TYPE http_requests_by_service counter\n");
            for (service, count) in service_counts {
                output.push_str(&format!(
                    "http_requests_by_service{{service=\"{}\"}} {}\n",
                    service, count
                ));
            }
        }

        Ok(output)
    }

    /// Export as JSON (for consumption by dashboards)
    pub fn export_json(&self) -> Result<serde_json::Value> {
        let histogram = self.collector.get_latency_histogram();
        let counters = self.collector.get_status_counters();
        let service_counts = self.collector.get_all_service_counts();

        let json = serde_json::json!({
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "requests": {
                "total": self.collector.get_request_count(),
                "in_flight": self.collector.get_in_flight_count(),
            },
            "status": {
                "success_2xx": counters.success_2xx,
                "client_error_4xx": counters.client_error_4xx,
                "server_error_5xx": counters.server_error_5xx,
            },
            "latency": {
                "min_ms": if histogram.count > 0 { histogram.min_ms } else { 0 },
                "max_ms": histogram.max_ms,
                "avg_ms": histogram.avg_ms,
                "p50_ms": histogram.p50_ms,
                "p95_ms": histogram.p95_ms,
                "p99_ms": histogram.p99_ms,
            },
            "services": service_counts.into_iter()
                .collect::<std::collections::HashMap<_, _>>(),
        });

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::RequestMetrics;

    // Traces to: FR-ROUTER-011 (Prometheus export)
    #[test]
    fn test_prometheus_export_basic() {
        let collector = MetricsCollector::new();
        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export().unwrap();

        assert!(output.contains("http_requests_total"));
        assert!(output.contains("http_requests_in_flight"));
        assert!(output.contains("http_requests_by_status"));
    }

    // Traces to: FR-ROUTER-011
    #[test]
    fn test_prometheus_export_format() {
        let collector = MetricsCollector::new();
        let exporter = PrometheusExporter::new(collector);
        let output = exporter.export().unwrap();

        // Check Prometheus format
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("http_requests_total"));
    }

    // Traces to: FR-ROUTER-011
    #[test]
    fn test_json_export() {
        let collector = MetricsCollector::new();
        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        let exporter = PrometheusExporter::new(collector);
        let json = exporter.export_json().unwrap();

        assert!(json.get("timestamp").is_some());
        assert!(json.get("requests").is_some());
        assert!(json.get("status").is_some());
        assert!(json.get("latency").is_some());
        assert!(json.get("services").is_some());
    }

    // Traces to: FR-ROUTER-011
    #[test]
    fn test_json_export_values() {
        let collector = MetricsCollector::new();
        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        let exporter = PrometheusExporter::new(collector);
        let json = exporter.export_json().unwrap();

        assert_eq!(json["requests"]["total"], 1);
        assert_eq!(json["requests"]["in_flight"], 0);
        assert_eq!(json["status"]["success_2xx"], 1);
        assert!(json["latency"]["avg_ms"] > 0.0);
    }
}
