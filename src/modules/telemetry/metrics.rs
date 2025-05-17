use axum::Router;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Initialize the Prometheus metrics exporter
pub fn init_prometheus_exporter(
    addr: SocketAddr,
) -> Result<PrometheusHandle, Box<dyn std::error::Error>> {
    let builder = PrometheusBuilder::new();

    // Configure Prometheus metrics
    let builder = builder
        .set_buckets_for_metric(
            Matcher::Full("intellirouter.http.latency".to_string()),
            &[
                1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
            ],
        )?
        .set_buckets_for_metric(
            Matcher::Full("intellirouter.llm.latency".to_string()),
            &[
                10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 30000.0, 60000.0,
            ],
        )?
        .set_buckets_for_metric(
            Matcher::Full("intellirouter.routing.decision_time".to_string()),
            &[
                0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0,
            ],
        )?;

    // Install the Prometheus metrics exporter
    let handle = builder.install_recorder()?;
    let handle_clone = handle.clone();

    // Start the Prometheus metrics server
    tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/metrics",
            axum::routing::get(|| async move {
                let metrics = handle_clone.render();
                axum::response::Response::builder()
                    .status(axum::http::StatusCode::OK)
                    .header("Content-Type", "text/plain")
                    .body(axum::body::Body::from(metrics))
                    .unwrap()
            }),
        );

        tracing::info!("Starting metrics server on {}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    Ok(handle)
}
