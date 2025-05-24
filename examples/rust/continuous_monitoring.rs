//! Continuous Monitoring Example
//!
//! This example demonstrates how to use the Continuous Improvement and Monitoring System
//! to monitor IntelliRouter components, collect metrics, and generate alerts.

use std::collections::HashMap;
use std::time::Duration;

use intellirouter::modules::monitoring::{
    Alert, AlertConfig, AlertManager, AlertSeverity, AlertStatus, AlertingSystem,
    ComponentHealthStatus, Dashboard, DashboardConfig, DashboardManager, DashboardPanel,
    DashboardView, DistributedTracing, FeedbackLoop, ImprovementSuggestion, LogConfig, LogFormat,
    LogLevel, LoggingSystem, Metric, MetricConfig, MetricsCollector, MetricsSystem,
    MonitoringConfig, MonitoringSystem, SpanContext, SuggestionPriority, SuggestionStatus, Tracer,
    TracingConfig, TracingSystem,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Continuous Monitoring Example");
    println!("=============================");

    // Create monitoring configuration
    let monitoring_config = MonitoringConfig {
        enabled: true,
        metrics_config: MetricConfig {
            enabled: true,
            endpoint: "127.0.0.1".to_string(),
            port: 9090,
            collection_interval_secs: 15,
            retention_days: 30,
            enable_prometheus: true,
            enable_opentelemetry: true,
            enable_dashboard: true,
        },
        log_config: LogConfig {
            enabled: true,
            level: LogLevel::Info,
            format: LogFormat::Json,
            file_path: Some(std::path::PathBuf::from("logs/intellirouter.log")),
            console_enabled: true,
            file_enabled: true,
            syslog_enabled: false,
            json_enabled: true,
            structured_enabled: true,
            rotation: Default::default(),
        },
        tracing_config: TracingConfig {
            enabled: true,
            sampling_rate: 0.1,
            enable_opentelemetry: true,
            enable_jaeger: true,
            enable_zipkin: false,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            zipkin_endpoint: None,
            service_name: "intellirouter".to_string(),
            environment: "development".to_string(),
        },
        alert_config: AlertConfig {
            enabled: true,
            check_interval_secs: 60,
            enable_email: false,
            enable_slack: true,
            enable_pagerduty: false,
            enable_webhook: false,
            email_config: None,
            slack_config: Some(Default::default()),
            pagerduty_config: None,
            webhook_config: None,
        },
        dashboard_config: DashboardConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 8080,
            title: "IntelliRouter Dashboard".to_string(),
            description: Some("Monitoring dashboard for IntelliRouter".to_string()),
            refresh_interval: 30,
            theme: "light".to_string(),
            logo_url: None,
            favicon_url: None,
            static_dir: std::path::PathBuf::from("dashboard/static"),
            templates_dir: std::path::PathBuf::from("dashboard/templates"),
            custom_css_url: None,
            custom_js_url: None,
        },
    };

    // Create monitoring system
    let mut monitoring_system = MonitoringSystem::new(monitoring_config);
    println!("Created monitoring system");

    // Initialize monitoring system
    monitoring_system.initialize().await?;
    println!("Initialized monitoring system");

    // Start monitoring system
    monitoring_system.start().await?;
    println!("Started monitoring system");

    // Get metrics system
    let metrics_system = monitoring_system.metrics_system();
    println!("Got metrics system");

    // Get metrics collector
    let metrics_collector = metrics_system.collector();
    println!("Got metrics collector");

    // Record some metrics
    let metric1 = Metric::new(
        "request_count",
        "Request Count",
        100.0,
        intellirouter::modules::monitoring::MetricType::Counter,
    )
    .with_tag("service", "router")
    .with_tag("endpoint", "/api/v1/chat")
    .with_unit("requests");

    let metric2 = Metric::new(
        "response_time",
        "Response Time",
        150.0,
        intellirouter::modules::monitoring::MetricType::Gauge,
    )
    .with_tag("service", "router")
    .with_tag("endpoint", "/api/v1/chat")
    .with_unit("ms");

    metrics_collector.record_metric(metric1).await?;
    metrics_collector.record_metric(metric2).await?;
    println!("Recorded metrics");

    // Get logging system
    let logging_system = monitoring_system.logging_system();
    println!("Got logging system");

    // Log some messages
    logging_system
        .info("This is an info message", "continuous_monitoring.rs")
        .await?;
    logging_system
        .warn("This is a warning message", "continuous_monitoring.rs")
        .await?;
    logging_system
        .error("This is an error message", "continuous_monitoring.rs")
        .await?;
    println!("Logged messages");

    // Get tracing system
    let tracing_system = monitoring_system.tracing_system();
    println!("Got tracing system");

    // Get tracer
    let tracer = tracing_system.tracer();
    println!("Got tracer");

    // Create a trace
    let trace_context = tracer.create_trace("example-trace").await?;
    println!("Created trace: {}", trace_context.trace_id);

    // Create a span
    let span_context = tracer
        .create_span(Some(&trace_context), "example-span", "SERVER")
        .await?;
    println!("Created span: {}", span_context.span_id);

    // Add span attributes
    tracer
        .add_span_attribute(&span_context, "service", "router")
        .await?;
    tracer
        .add_span_attribute(&span_context, "endpoint", "/api/v1/chat")
        .await?;
    println!("Added span attributes");

    // Add span event
    let event = intellirouter::modules::monitoring::SpanEvent::new("example-event")
        .with_attribute("key", "value");
    tracer.add_span_event(&span_context, event).await?;
    println!("Added span event");

    // End span
    tracer.end_span(&span_context).await?;
    println!("Ended span");

    // Get alerting system
    let alerting_system = monitoring_system.alerting_system();
    println!("Got alerting system");

    // Get alert manager
    let alert_manager = alerting_system.manager();
    println!("Got alert manager");

    // Create an alert rule
    let alert_rule = intellirouter::modules::monitoring::AlertRule::new(
        "high_latency",
        "High Latency",
        "Alert when latency exceeds 200ms",
        AlertSeverity::Warning,
        "response_time > 200",
    )
    .with_for_duration(Duration::from_secs(60))
    .with_label("service", "router")
    .with_label("endpoint", "/api/v1/chat")
    .with_runbook_url("https://example.com/runbooks/high_latency");

    alert_manager.add_rule(alert_rule).await?;
    println!("Added alert rule");

    // Trigger an alert
    let alert = Alert::new(
        "alert-1",
        "High Latency Detected",
        "Response time exceeded 200ms",
        AlertSeverity::Warning,
        "monitoring_system",
    )
    .with_label("service", "router")
    .with_label("endpoint", "/api/v1/chat")
    .with_runbook_url("https://example.com/runbooks/high_latency");

    alert_manager.trigger_alert(alert).await?;
    println!("Triggered alert");

    // Get dashboard system
    let dashboard_system = monitoring_system.dashboard_system();
    println!("Got dashboard system");

    // Create a dashboard
    let dashboard = Dashboard::new("main", "IntelliRouter Dashboard")
        .with_description("Main dashboard for IntelliRouter")
        .with_panel(
            DashboardPanel::new("request_count", "Request Count", "line-chart", "metrics")
                .with_query("request_count")
                .with_dimensions(6, 6)
                .with_position(0, 0),
        )
        .with_panel(
            DashboardPanel::new("response_time", "Response Time", "gauge", "metrics")
                .with_query("response_time")
                .with_dimensions(6, 6)
                .with_position(6, 0),
        )
        .with_view(
            DashboardView::new("main", "Main View")
                .with_description("Main dashboard view")
                .with_panel("request_count")
                .with_panel("response_time")
                .with_layout("grid")
                .with_theme("light"),
        )
        .with_default_view("main");

    dashboard_system.add_dashboard(dashboard).await?;
    println!("Added dashboard");

    // Get improvement system
    let improvement_system = monitoring_system.improvement_system();
    println!("Got improvement system");

    // Create a feedback loop
    let mut feedback_loop = FeedbackLoop::new(
        "performance",
        "Performance Feedback Loop",
        "Analyzes performance metrics and suggests improvements",
        Duration::from_secs(3600),
    );
    feedback_loop.run();

    improvement_system.add_feedback_loop(feedback_loop).await?;
    println!("Added feedback loop");

    // Create a suggestion
    let suggestion = ImprovementSuggestion::new(
        "suggestion-1",
        "Optimize Router Caching",
        "Implement a more efficient caching strategy for the router to reduce latency",
        SuggestionPriority::High,
    )
    .with_tag("performance")
    .with_tag("latency")
    .with_affected_task("optimize-router")
    .with_implementation_step("Analyze current caching strategy")
    .with_implementation_step("Research alternative caching algorithms")
    .with_implementation_step("Implement and test new caching strategy")
    .with_expected_benefit("Reduced latency by 30%")
    .with_expected_benefit("Improved throughput by 20%")
    .with_potential_risk("Increased memory usage");

    improvement_system.add_suggestion(suggestion).await?;
    println!("Added suggestion");

    // Run a health check
    let health_status = monitoring_system.health_check().await?;
    println!("\nHealth Check Results:");
    println!(
        "Overall Status: {}",
        if health_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Metrics System: {}",
        if health_status.metrics_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Logging System: {}",
        if health_status.logging_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Tracing System: {}",
        if health_status.tracing_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Alerting System: {}",
        if health_status.alerting_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Dashboard System: {}",
        if health_status.dashboard_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );
    println!(
        "Improvement System: {}",
        if health_status.improvement_status.healthy {
            "Healthy"
        } else {
            "Unhealthy"
        }
    );

    // Stop monitoring system
    monitoring_system.stop().await?;
    println!("\nStopped monitoring system");

    println!("\nContinuous monitoring example completed successfully!");
    Ok(())
}
