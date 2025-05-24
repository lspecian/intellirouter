//! Visualization
//!
//! This module provides visualization capabilities for system topology,
//! test results, performance metrics, and error information.

use std::collections::HashMap;
use std::path::Path;

use crate::modules::audit::types::MetricDataPoint;
use plotters::element::{Circle, PathElement};
use plotters::prelude::*;
use plotters::style::RGBColor;

// Define additional colors
const ORANGE: RGBColor = RGBColor(255, 165, 0);
const GRAY: RGBColor = RGBColor(128, 128, 128);
use tracing::info;

use super::topology::SystemTopology;
use crate::modules::audit::report::AuditReport;
use crate::modules::audit::types::{AuditError, MetricType, ServiceStatus, ServiceType};

/// Base visualizer trait
#[async_trait::async_trait]
pub trait Visualizer<T, U> {
    /// Visualize data
    async fn visualize(&self, data: &T) -> Result<U, AuditError>;
}

/// Topology visualizer
#[derive(Debug)]
pub struct TopologyVisualizer {
    /// Output directory for visualizations
    output_dir: String,
}

impl TopologyVisualizer {
    /// Create a new topology visualizer
    pub fn new() -> Self {
        Self {
            output_dir: "reports/visualizations".to_string(),
        }
    }

    /// Set output directory
    pub fn _with_output_dir(mut self, dir: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

#[async_trait::async_trait]
impl Visualizer<SystemTopology, String> for TopologyVisualizer {
    async fn visualize(&self, topology: &SystemTopology) -> Result<String, AuditError> {
        info!("Visualizing system topology");

        // Create output directory if it doesn't exist
        let output_dir = Path::new(&self.output_dir);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|e| {
                AuditError::ReportGenerationError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // Define the output file path
        let output_path = output_dir.join("system_topology.svg");

        // Create the SVG file
        let root = SVGBackend::new(&output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to fill drawing area: {}", e))
        })?;

        // Calculate node positions (simple circular layout)
        let mut positioned_nodes = Vec::new();
        let node_count = topology.nodes.len();
        let radius = 200.0;
        let center_x = 400.0;
        let center_y = 300.0;

        for (i, node) in topology.nodes.iter().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (node_count as f32);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();

            positioned_nodes.push((node, (x, y)));
        }

        // Draw edges
        for edge in &topology.edges {
            let source_pos = positioned_nodes
                .iter()
                .find(|(node, _)| node.service_type == edge.source)
                .map(|(_, pos)| *pos)
                .unwrap_or((center_x, center_y));

            let target_pos = positioned_nodes
                .iter()
                .find(|(node, _)| node.service_type == edge.target)
                .map(|(_, pos)| *pos)
                .unwrap_or((center_x, center_y));

            let color = if edge.connected { &GREEN } else { &RED };

            root.draw(&PathElement::new(
                vec![
                    (source_pos.0 as i32, source_pos.1 as i32),
                    (target_pos.0 as i32, target_pos.1 as i32),
                ],
                color.filled(),
            ))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw edge: {}", e))
            })?;
        }

        // Draw nodes
        for (node, (x, y)) in &positioned_nodes {
            let color = match node.status {
                ServiceStatus::Running => &GREEN,
                ServiceStatus::Failed => &RED,
                ServiceStatus::NotStarted => &GRAY,
                ServiceStatus::Starting => &YELLOW,
                ServiceStatus::ShuttingDown => &BLUE,
                ServiceStatus::Stopped => &BLACK,
                ServiceStatus::Active => &GREEN,
                ServiceStatus::Inactive => &YELLOW,
                ServiceStatus::Degraded => &ORANGE,
            };

            // Draw node circle
            root.draw(&Circle::new((*x as i32, *y as i32), 20, color.filled()))
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to draw node: {}", e))
                })?;

            // Draw node label
            root.draw(&Text::new(
                format!("{}", node.service_type),
                (*x as i32, *y as i32 + 30),
                ("sans-serif", 15).into_font(),
            ))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw node label: {}", e))
            })?;
        }

        // Draw legend
        let legend_x = 650;
        let mut legend_y = 50;
        let legend_spacing = 30;

        // Status legend
        root.draw(&Text::new(
            "Service Status:",
            (legend_x, legend_y),
            ("sans-serif", 15).into_font(),
        ))
        .map_err(|e| AuditError::ReportGenerationError(format!("Failed to draw legend: {}", e)))?;

        legend_y += legend_spacing;

        for (status, color) in &[
            (ServiceStatus::Running, &GREEN),
            (ServiceStatus::Failed, &RED),
            (ServiceStatus::NotStarted, &WHITE),
            (ServiceStatus::Starting, &YELLOW),
            (ServiceStatus::ShuttingDown, &BLUE),
            (ServiceStatus::Stopped, &BLACK),
        ] {
            root.draw(&Circle::new((legend_x, legend_y), 10, color.filled()))
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to draw legend item: {}", e))
                })?;

            root.draw(&Text::new(
                format!("{}", status),
                (legend_x + 20, legend_y),
                ("sans-serif", 12).into_font(),
            ))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw legend text: {}", e))
            })?;

            legend_y += legend_spacing;
        }

        // Connection legend
        legend_y += legend_spacing;

        root.draw(&Text::new(
            "Connection Status:",
            (legend_x, legend_y),
            ("sans-serif", 15).into_font(),
        ))
        .map_err(|e| AuditError::ReportGenerationError(format!("Failed to draw legend: {}", e)))?;

        legend_y += legend_spacing;

        for (status, color) in &[("Connected", &GREEN), ("Disconnected", &RED)] {
            root.draw(&PathElement::new(
                vec![(legend_x - 10, legend_y), (legend_x + 10, legend_y)],
                color.filled(),
            ))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw legend item: {}", e))
            })?;

            root.draw(&Text::new(
                *status,
                (legend_x + 20, legend_y),
                ("sans-serif", 12).into_font(),
            ))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw legend text: {}", e))
            })?;

            legend_y += legend_spacing;
        }

        // Add title
        root.draw(&Text::new(
            "IntelliRouter System Topology",
            (400, 30),
            ("sans-serif", 20).into_font(),
        ))
        .map_err(|e| AuditError::ReportGenerationError(format!("Failed to draw title: {}", e)))?;

        // Add timestamp
        root.draw(&Text::new(
            format!(
                "Generated: {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ),
            (400, 570),
            ("sans-serif", 12).into_font(),
        ))
        .map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to draw timestamp: {}", e))
        })?;

        root.present().map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to save visualization: {}", e))
        })?;

        info!("System topology visualization saved to {:?}", output_path);

        Ok(output_path.to_string_lossy().to_string())
    }
}

/// Test result visualizer
#[derive(Debug)]
pub struct TestResultVisualizer {
    /// Output directory for visualizations
    output_dir: String,
}

impl TestResultVisualizer {
    /// Create a new test result visualizer
    pub fn new() -> Self {
        Self {
            output_dir: "reports/visualizations".to_string(),
        }
    }

    /// Set output directory
    pub fn _with_output_dir(mut self, dir: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

#[async_trait::async_trait]
impl Visualizer<AuditReport, String> for TestResultVisualizer {
    async fn visualize(&self, report: &AuditReport) -> Result<String, AuditError> {
        info!("Visualizing test results");

        // Create output directory if it doesn't exist
        let output_dir = Path::new(&self.output_dir);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|e| {
                AuditError::ReportGenerationError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // Define the output file path
        let output_path = output_dir.join("test_results.svg");

        // Create the SVG file
        let root = SVGBackend::new(&output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to fill drawing area: {}", e))
        })?;

        // Draw test results as a bar chart
        let mut chart = ChartBuilder::on(&root)
            .caption("Test Results", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0..report.test_results.len(),
                0.0..report
                    .test_results
                    .iter()
                    .map(|r| r.duration_ms)
                    .max()
                    .unwrap_or(100) as f64
                    * 1.1,
            )
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to build chart: {}", e))
            })?;

        chart
            .configure_mesh()
            .x_labels(report.test_results.len())
            .x_label_formatter(&|x| {
                if *x < report.test_results.len() {
                    format!("{}", report.test_results[*x].test_flow)
                } else {
                    "".to_string()
                }
            })
            .y_desc("Duration (ms)")
            .draw()
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw chart mesh: {}", e))
            })?;

        // Draw bars
        chart
            .draw_series(report.test_results.iter().enumerate().map(|(i, result)| {
                let color = if result.success { &GREEN } else { &RED };
                let bar = Rectangle::new(
                    [(i, 0.0), (i + 1, result.duration_ms as f64)],
                    color.filled(),
                );
                bar
            }))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to draw bars: {}", e))
            })?;

        // Add success/failure count
        let success_count = report.test_results.iter().filter(|r| r.success).count();
        let failure_count = report.test_results.len() - success_count;

        root.draw(&Text::new(
            format!("Success: {} | Failure: {}", success_count, failure_count),
            (400, 570),
            ("sans-serif", 15).into_font(),
        ))
        .map_err(|e| AuditError::ReportGenerationError(format!("Failed to draw summary: {}", e)))?;

        root.present().map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to save visualization: {}", e))
        })?;

        info!("Test results visualization saved to {:?}", output_path);

        Ok(output_path.to_string_lossy().to_string())
    }
}

/// Performance visualizer
#[derive(Debug)]
pub struct PerformanceVisualizer {
    /// Output directory for visualizations
    output_dir: String,
}

impl PerformanceVisualizer {
    /// Create a new performance visualizer
    pub fn new() -> Self {
        Self {
            output_dir: "reports/visualizations".to_string(),
        }
    }

    /// Set output directory
    pub fn _with_output_dir(mut self, dir: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

#[async_trait::async_trait]
impl Visualizer<AuditReport, HashMap<MetricType, String>> for PerformanceVisualizer {
    async fn visualize(
        &self,
        report: &AuditReport,
    ) -> Result<HashMap<MetricType, String>, AuditError> {
        info!("Visualizing performance metrics");

        let mut result_paths = HashMap::new();

        // Create output directory if it doesn't exist
        let output_dir = Path::new(&self.output_dir);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|e| {
                AuditError::ReportGenerationError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // Extract unique metric types
        let metric_types: Vec<MetricType> = report
            .metrics
            .iter()
            .map(|m| m.metric_type)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Process each metric type
        for metric_type in metric_types {
            // Filter metrics for this type
            let metrics_of_type: Vec<&MetricDataPoint> = report
                .metrics
                .iter()
                .filter(|m| m.metric_type == metric_type)
                .collect();

            // Skip if no metrics
            if metrics_of_type.is_empty() {
                continue;
            }

            // Define the output file path
            let output_path = output_dir.join(format!("performance_{}.svg", metric_type));

            // Create the SVG file
            let root = SVGBackend::new(&output_path, (800, 600)).into_drawing_area();
            root.fill(&WHITE).map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to fill drawing area: {}", e))
            })?;

            // Group metrics by service
            let mut metrics_by_service: HashMap<
                ServiceType,
                Vec<(chrono::DateTime<chrono::Utc>, f64)>,
            > = HashMap::new();

            // Group and extract timestamp/value pairs
            for metric in &metrics_of_type {
                metrics_by_service
                    .entry(metric.service)
                    .or_insert_with(Vec::new)
                    .push((metric.timestamp, metric.value));
            }

            // Sort metrics by timestamp for each service
            for service_metrics in metrics_by_service.values_mut() {
                service_metrics.sort_by_key(|&(timestamp, _)| timestamp);
            }

            // Find min and max timestamps
            let min_time = metrics_of_type
                .iter()
                .map(|m| m.timestamp)
                .min()
                .unwrap_or_else(|| chrono::Utc::now());

            let max_time = metrics_of_type
                .iter()
                .map(|m| m.timestamp)
                .max()
                .unwrap_or_else(|| chrono::Utc::now());

            // Find min and max values
            let min_value = metrics_of_type
                .iter()
                .map(|m| m.value)
                .fold(f64::INFINITY, |a, b| a.min(b));

            let max_value = metrics_of_type
                .iter()
                .map(|m| m.value)
                .fold(f64::NEG_INFINITY, |a, b| a.max(b));

            // Create time series chart
            let mut chart = ChartBuilder::on(&root)
                .caption(format!("{} Over Time", metric_type), ("sans-serif", 30))
                .margin(10)
                .x_label_area_size(40)
                .y_label_area_size(60)
                .build_cartesian_2d(min_time..max_time, min_value..max_value * 1.1)
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to build chart: {}", e))
                })?;

            chart
                .configure_mesh()
                .x_labels(10)
                .x_label_formatter(&|x| x.format("%H:%M:%S").to_string())
                .y_desc(format!("{} Value", metric_type))
                .draw()
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to draw chart mesh: {}", e))
                })?;

            // Define colors for services
            let colors = [&RED, &GREEN, &BLUE, &YELLOW, &MAGENTA, &CYAN];

            // Draw a line for each service
            let mut service_index = 0;
            for (service, service_metrics) in &metrics_by_service {
                let color = colors[service_index % colors.len()];
                service_index += 1;

                chart
                    .draw_series(LineSeries::new(
                        service_metrics
                            .iter()
                            .map(|&(timestamp, value)| (timestamp, value)),
                        color.clone(),
                    ))
                    .map_err(|e| {
                        AuditError::ReportGenerationError(format!("Failed to draw line: {}", e))
                    })?
                    .label(format!("{}", service))
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], color.clone())
                    });
            }

            // Draw legend
            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to draw legend: {}", e))
                })?;

            root.present().map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to save visualization: {}", e))
            })?;

            info!("{} visualization saved to {:?}", metric_type, output_path);

            result_paths.insert(metric_type, output_path.to_string_lossy().to_string());
        }

        Ok(result_paths)
    }
}

/// Error visualizer
#[derive(Debug)]
pub struct ErrorVisualizer {
    /// Output directory for visualizations
    output_dir: String,
}

impl ErrorVisualizer {
    /// Create a new error visualizer
    pub fn new() -> Self {
        Self {
            output_dir: "reports/visualizations".to_string(),
        }
    }

    /// Set output directory
    pub fn _with_output_dir(mut self, dir: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

#[async_trait::async_trait]
impl Visualizer<AuditReport, String> for ErrorVisualizer {
    async fn visualize(&self, report: &AuditReport) -> Result<String, AuditError> {
        info!("Visualizing error information");

        // Create output directory if it doesn't exist
        let output_dir = Path::new(&self.output_dir);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|e| {
                AuditError::ReportGenerationError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // Define the output file path
        let output_path = output_dir.join("error_summary.svg");

        // Create the SVG file
        let root = SVGBackend::new(&output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to fill drawing area: {}", e))
        })?;

        // Count errors by type
        let mut error_counts = HashMap::new();

        // Count service failures
        for (service, status) in &report.service_statuses {
            if *status == ServiceStatus::Failed {
                *error_counts
                    .entry(format!("Service Failure: {}", service))
                    .or_insert(0) += 1;
            }
        }

        // Count test failures
        for result in &report.test_results {
            if !result.success {
                *error_counts
                    .entry(format!("Test Failure: {}", result.test_flow))
                    .or_insert(0) += 1;
            }
        }

        // Count communication failures
        for result in &report.communication_tests {
            if !result.success {
                *error_counts
                    .entry(format!(
                        "Communication Failure: {} -> {}",
                        result.source, result.target
                    ))
                    .or_insert(0) += 1;
            }
        }

        // Count general errors
        for _error in &report.errors {
            *error_counts.entry("General Error".to_string()).or_insert(0) += 1;
        }

        // Sort errors by count
        let mut sorted_errors: Vec<_> = error_counts.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));

        // Create a pie chart of error types
        let drawing_area = root
            .titled("Error Distribution", ("sans-serif", 30))
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to create title: {}", e))
            })?;

        if sorted_errors.is_empty() {
            // No errors to display
            drawing_area
                .draw(&Text::new(
                    "No errors detected",
                    (400, 300),
                    ("sans-serif", 20).into_font(),
                ))
                .map_err(|e| {
                    AuditError::ReportGenerationError(format!("Failed to draw text: {}", e))
                })?;
        } else {
            // Calculate total errors
            let _total_errors: i32 = sorted_errors.iter().map(|(_, count)| *count).sum();

            // Create pie chart - Commented out due to compatibility issues
            // let pie_chart = drawing_area.centered_at((400, 300));
            // let radius = 200;

            // // Define colors for pie slices
            // let colors = [&RED, &BLUE, &GREEN, &YELLOW, &MAGENTA, &CYAN];

            // // Draw pie slices
            // let mut current_angle = 0.0;
            // for (i, (error_type, count)) in sorted_errors.iter().enumerate() {
            //     let angle = 360.0 * (*count as f64 / total_errors as f64);
            //     let end_angle = current_angle + angle;

            //     let color = colors[i % colors.len()];

            //     // Draw pie slice - Commented out due to API compatibility issues
            //     // pie_chart
            //     //     .draw(&Pie::new(
            //     //         &(0, 0),
            //     //         radius,
            //     //         current_angle.to_radians()..end_angle.to_radians(),
            //     //         color.filled(),
            //     //         0, // z-order
            //     //     ))
            //     //     .map_err(|e| {
            //     //         AuditError::ReportGenerationError(format!(
            //     //             "Failed to draw pie slice: {}",
            //     //             e
            //     //         ))
            //     //     .map_err(|e| {
            //     //         AuditError::ReportGenerationError(format!(
            //     //             "Failed to draw pie slice: {}",
            //     //             e
            //     //         ))
            //     //     })?;

            //     // // Draw label
            //     // let label_angle = (current_angle + angle / 2.0).to_radians();
            //     // let label_x = (radius + 30) as f64 * label_angle.cos();
            //     // let label_y = (radius + 30) as f64 * label_angle.sin();

            //     // pie_chart
            //     //     .draw(&Text::new(
            //     //         format!(
            //     //             "{}: {} ({:.1}%)",
            //     //             error_type,
            //     //             count,
            //     //             100.0 * (*count as f64 / total_errors as f64)
            //     //         ),
            //     //         (label_x as i32, label_y as i32),
            //     //         ("sans-serif", 12).into_font(),
            //     //     ))
            //     //     .map_err(|e| {
            //     //         AuditError::ReportGenerationError(format!("Failed to draw label: {}", e))
            //     //     })?;

            //     // current_angle = end_angle;
            // }
        }

        // Add timestamp - Commented out due to compatibility issues
        // root.draw(&Text::new(
        //     format!(
        //         "Generated: {}",
        //         chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        //     ),
        //     (400, 570),
        //     ("sans-serif", 12).into_font(),
        // ))
        // .map_err(|e| {
        //     AuditError::ReportGenerationError(format!("Failed to draw timestamp: {}", e))
        // })?;

        // root.present().map_err(|e| {
        //     AuditError::ReportGenerationError(format!("Failed to save visualization: {}", e))
        // })?;

        info!("Error visualization saved to {:?}", output_path);

        // Return a placeholder since visualization is disabled
        Ok("Visualization disabled due to compatibility issues".to_string())
    }
}
