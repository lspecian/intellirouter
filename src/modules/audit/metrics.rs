//! Metrics Collector
//!
//! This module is responsible for collecting metrics on system performance
//! during the audit process.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::report::AuditReport;
use super::types::{AuditError, MetricDataPoint, MetricType, MetricsConfig, ServiceType};

/// Metrics Collector
#[derive(Debug)]
pub struct MetricsCollector {
    /// Metrics configuration
    config: MetricsConfig,
    /// HTTP client for API requests
    client: Client,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig, report: Arc<RwLock<AuditReport>>) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            report,
        }
    }

    /// Collect metrics
    pub async fn collect_metrics(&self) -> Result<(), AuditError> {
        if !self.config.collect_metrics {
            info!("Metrics collection is disabled");
            return Ok(());
        }

        info!("Starting metrics collection");

        let start_time = Instant::now();
        let collection_duration = Duration::from_secs(self.config.collection_duration_secs);
        let collection_interval = Duration::from_millis(self.config.collection_interval_ms);

        // Define the services to collect metrics from
        let services = vec![
            ServiceType::Router,
            ServiceType::ChainEngine,
            ServiceType::RagManager,
            ServiceType::PersonaLayer,
        ];

        // Collect metrics at regular intervals
        while start_time.elapsed() < collection_duration {
            for service in &services {
                for metric_type in &self.config.metric_types {
                    match self.collect_metric(*service, *metric_type).await {
                        Ok(data_point) => {
                            // Add metric data point to the report
                            let mut report = self.report.write().await;
                            report.add_metric(data_point);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to collect {} metric for {}: {}",
                                metric_type, service, e
                            );
                        }
                    }
                }
            }

            // Wait for the next collection interval
            sleep(collection_interval).await;
        }

        info!("Metrics collection completed");

        // Analyze the collected metrics
        self.analyze_metrics().await?;

        Ok(())
    }

    /// Collect a specific metric for a service
    async fn collect_metric(
        &self,
        service: ServiceType,
        metric_type: MetricType,
    ) -> Result<MetricDataPoint, AuditError> {
        // Get the service host and port
        let (host, port) = match service {
            ServiceType::Router => ("router", 8080),
            ServiceType::ChainEngine => ("orchestrator", 8080),
            ServiceType::RagManager => ("rag-injector", 8080),
            ServiceType::PersonaLayer => ("summarizer", 8080),
            _ => {
                return Err(AuditError::MetricsCollectionError(format!(
                    "Metrics collection not supported for service: {}",
                    service
                )));
            }
        };

        // Get the metric value
        let value = match metric_type {
            MetricType::Latency => self.collect_latency_metric(host, port).await?,
            MetricType::Throughput => self.collect_throughput_metric(host, port).await?,
            MetricType::ErrorRate => self.collect_error_rate_metric(host, port).await?,
            MetricType::ResourceUsage => self.collect_resource_usage_metric(host, port).await?,
        };

        Ok(MetricDataPoint {
            metric_type,
            service,
            value,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Collect latency metric
    async fn collect_latency_metric(&self, host: &str, port: u16) -> Result<f64, AuditError> {
        // Measure the time it takes to get a response from the health endpoint
        let url = format!("http://{}:{}/health", host, port);

        let start_time = Instant::now();
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;
        let elapsed = start_time.elapsed();

        if !response.status().is_success() {
            return Err(AuditError::MetricsCollectionError(format!(
                "Failed to get health status from {}: status code {}",
                url,
                response.status()
            )));
        }

        // Return the latency in milliseconds
        Ok(elapsed.as_secs_f64() * 1000.0)
    }

    /// Collect throughput metric
    async fn collect_throughput_metric(&self, host: &str, port: u16) -> Result<f64, AuditError> {
        // Get the throughput from the diagnostics endpoint
        let url = format!("http://{}:{}/diagnostics", host, port);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::MetricsCollectionError(format!(
                "Failed to get diagnostics from {}: status code {}",
                url,
                response.status()
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Try to extract throughput from the diagnostics
        if let Some(diagnostics) = body.get("diagnostics") {
            if let Some(throughput) = diagnostics.get("throughput") {
                if let Some(requests_per_second) = throughput.get("requests_per_second") {
                    if let Some(value) = requests_per_second.as_f64() {
                        return Ok(value);
                    }
                }
            }
        }

        // If we couldn't find throughput in the diagnostics, return a default value
        Ok(0.0)
    }

    /// Collect error rate metric
    async fn collect_error_rate_metric(&self, host: &str, port: u16) -> Result<f64, AuditError> {
        // Get the error rate from the diagnostics endpoint
        let url = format!("http://{}:{}/diagnostics", host, port);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::MetricsCollectionError(format!(
                "Failed to get diagnostics from {}: status code {}",
                url,
                response.status()
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Try to extract error rate from the diagnostics
        if let Some(diagnostics) = body.get("diagnostics") {
            if let Some(error_rate) = diagnostics.get("error_rate") {
                if let Some(value) = error_rate.as_f64() {
                    return Ok(value);
                }
            }
        }

        // If we couldn't find error rate in the diagnostics, return a default value
        Ok(0.0)
    }

    /// Collect resource usage metric
    async fn collect_resource_usage_metric(
        &self,
        host: &str,
        port: u16,
    ) -> Result<f64, AuditError> {
        // Get the resource usage from the diagnostics endpoint
        let url = format!("http://{}:{}/diagnostics", host, port);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::MetricsCollectionError(format!(
                "Failed to get diagnostics from {}: status code {}",
                url,
                response.status()
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Try to extract CPU usage from the resources
        if let Some(resources) = body.get("resources") {
            if let Some(cpu_percent) = resources.get("cpu_percent") {
                if let Some(value) = cpu_percent.as_f64() {
                    return Ok(value);
                }
            }
        }

        // If we couldn't find CPU usage in the resources, return a default value
        Ok(0.0)
    }

    /// Analyze the collected metrics
    async fn analyze_metrics(&self) -> Result<(), AuditError> {
        info!("Analyzing collected metrics");

        let report = self.report.read().await;
        let metrics = report.get_metrics();

        if metrics.is_empty() {
            warn!("No metrics collected, skipping analysis");
            return Ok(());
        }

        // Calculate average latency per service
        let mut latency_by_service: HashMap<ServiceType, Vec<f64>> = HashMap::new();
        for metric in metrics
            .iter()
            .filter(|m| m.metric_type == MetricType::Latency)
        {
            latency_by_service
                .entry(metric.service)
                .or_default()
                .push(metric.value);
        }

        let mut avg_latency_by_service: HashMap<ServiceType, f64> = HashMap::new();
        for (service, latencies) in &latency_by_service {
            if !latencies.is_empty() {
                let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
                avg_latency_by_service.insert(*service, avg_latency);

                info!("Average latency for {}: {:.2} ms", service, avg_latency);
            }
        }

        // Calculate average throughput per service
        let mut throughput_by_service: HashMap<ServiceType, Vec<f64>> = HashMap::new();
        for metric in metrics
            .iter()
            .filter(|m| m.metric_type == MetricType::Throughput)
        {
            throughput_by_service
                .entry(metric.service)
                .or_default()
                .push(metric.value);
        }

        let mut avg_throughput_by_service: HashMap<ServiceType, f64> = HashMap::new();
        for (service, throughputs) in &throughput_by_service {
            if !throughputs.is_empty() {
                let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
                avg_throughput_by_service.insert(*service, avg_throughput);

                info!(
                    "Average throughput for {}: {:.2} requests/sec",
                    service, avg_throughput
                );
            }
        }

        // Calculate average error rate per service
        let mut error_rate_by_service: HashMap<ServiceType, Vec<f64>> = HashMap::new();
        for metric in metrics
            .iter()
            .filter(|m| m.metric_type == MetricType::ErrorRate)
        {
            error_rate_by_service
                .entry(metric.service)
                .or_default()
                .push(metric.value);
        }

        let mut avg_error_rate_by_service: HashMap<ServiceType, f64> = HashMap::new();
        for (service, error_rates) in &error_rate_by_service {
            if !error_rates.is_empty() {
                let avg_error_rate = error_rates.iter().sum::<f64>() / error_rates.len() as f64;
                avg_error_rate_by_service.insert(*service, avg_error_rate);

                info!(
                    "Average error rate for {}: {:.2}%",
                    service,
                    avg_error_rate * 100.0
                );
            }
        }

        // Calculate average resource usage per service
        let mut resource_usage_by_service: HashMap<ServiceType, Vec<f64>> = HashMap::new();
        for metric in metrics
            .iter()
            .filter(|m| m.metric_type == MetricType::ResourceUsage)
        {
            resource_usage_by_service
                .entry(metric.service)
                .or_default()
                .push(metric.value);
        }

        let mut avg_resource_usage_by_service: HashMap<ServiceType, f64> = HashMap::new();
        for (service, resource_usages) in &resource_usage_by_service {
            if !resource_usages.is_empty() {
                let avg_resource_usage =
                    resource_usages.iter().sum::<f64>() / resource_usages.len() as f64;
                avg_resource_usage_by_service.insert(*service, avg_resource_usage);

                info!(
                    "Average CPU usage for {}: {:.2}%",
                    service, avg_resource_usage
                );
            }
        }

        // Add metric analysis to the report
        let mut report = self.report.write().await;

        // Add average latency
        for (service, avg_latency) in avg_latency_by_service {
            report.add_metric_analysis(
                service,
                MetricType::Latency,
                avg_latency,
                format!("Average latency: {:.2} ms", avg_latency),
            );
        }

        // Add average throughput
        for (service, avg_throughput) in avg_throughput_by_service {
            report.add_metric_analysis(
                service,
                MetricType::Throughput,
                avg_throughput,
                format!("Average throughput: {:.2} requests/sec", avg_throughput),
            );
        }

        // Add average error rate
        for (service, avg_error_rate) in avg_error_rate_by_service {
            report.add_metric_analysis(
                service,
                MetricType::ErrorRate,
                avg_error_rate,
                format!("Average error rate: {:.2}%", avg_error_rate * 100.0),
            );
        }

        // Add average resource usage
        for (service, avg_resource_usage) in avg_resource_usage_by_service {
            report.add_metric_analysis(
                service,
                MetricType::ResourceUsage,
                avg_resource_usage,
                format!("Average CPU usage: {:.2}%", avg_resource_usage),
            );
        }

        info!("Metrics analysis completed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let collector = MetricsCollector::new(config, report);

        assert!(collector.config.collect_metrics);
        assert_eq!(collector.config.collection_interval_ms, 1000);
        assert_eq!(collector.config.collection_duration_secs, 60);
        assert_eq!(collector.config.metric_types.len(), 4);
    }
}
