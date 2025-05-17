pub mod cost;
pub mod metrics;
pub mod middleware;
pub mod telemetry;
pub mod tests;

use std::net::SocketAddr;
use std::sync::Arc;

pub use cost::CostCalculator;
pub use middleware::telemetry_middleware;
pub use telemetry::{LlmCallMetrics, RoutingMetrics, TelemetryManager};

/// Initialize the telemetry module
pub fn init_telemetry(
    service_name: &str,
    environment: &str,
    version: &str,
    metrics_addr: SocketAddr,
) -> Result<Arc<TelemetryManager>, Box<dyn std::error::Error>> {
    // Set up logging
    TelemetryManager::setup_logging()?;

    // Initialize metrics exporter
    metrics::init_prometheus_exporter(metrics_addr)?;

    // Create telemetry manager
    let telemetry = Arc::new(TelemetryManager::new(
        service_name.to_string(),
        environment.to_string(),
        version.to_string(),
    ));

    Ok(telemetry)
}

/// Create a cost calculator
pub fn create_cost_calculator() -> Arc<CostCalculator> {
    Arc::new(CostCalculator::new())
}
