//! Benchmark result reporters
//!
//! This module provides functionality for generating reports and visualizations from benchmark results.

use crate::framework::metrics::{MetricsStorage, PerformanceRegression};
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Benchmark reporter for generating reports and visualizations
pub struct BenchmarkReporter {
    /// Metrics storage
    metrics: MetricsStorage,
    /// Output directory for reports
    output_dir: PathBuf,
}

impl BenchmarkReporter {
    /// Create a new benchmark reporter
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(metrics_dir: P, output_dir: Q) -> io::Result<Self> {
        let metrics = MetricsStorage::new(metrics_dir)?;
        let output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&output_dir)?;

        Ok(Self {
            metrics,
            output_dir,
        })
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> io::Result<PathBuf> {
        let report_path = self.output_dir.join("performance_report.md");
        let mut file = File::create(&report_path)?;

        writeln!(file, "# IntelliRouter Performance Report")?;
        writeln!(file, "\nGenerated at: {}\n", Utc::now().to_rfc3339())?;

        // Summary section
        writeln!(file, "## Summary\n")?;
        let components = self.metrics.get_components();
        writeln!(file, "Components benchmarked: {}", components.len())?;

        let mut total_benchmarks = 0;
        for component in &components {
            if let Some(benchmarks) = self.metrics.get_benchmarks(component) {
                total_benchmarks += benchmarks.len();
            }
        }
        writeln!(file, "Total benchmarks: {}\n", total_benchmarks)?;

        // Performance regressions
        let regressions = self.metrics.detect_regressions(0.05); // 5% threshold
        if !regressions.is_empty() {
            writeln!(file, "## Performance Regressions\n")?;
            writeln!(
                file,
                "| Component | Benchmark | Previous | Current | Change | Type |"
            )?;
            writeln!(
                file,
                "|-----------|-----------|----------|---------|--------|------|"
            )?;

            for regression in &regressions {
                writeln!(
                    file,
                    "| {} | {} | {:.2} | {:.2} | {:.2}% | {:?} |",
                    regression.component,
                    regression.benchmark,
                    regression.previous_value,
                    regression.current_value,
                    regression.change_percentage,
                    regression.regression_type
                )?;
            }
            writeln!(file)?;
        }

        // Component details
        writeln!(file, "## Component Details\n")?;

        for component in &components {
            writeln!(file, "### {}\n", component)?;

            if let Some(benchmarks) = self.metrics.get_benchmarks(component) {
                for benchmark in benchmarks {
                    if let Some(results) = self.metrics.get_benchmark_results(component, benchmark)
                    {
                        if results.is_empty() {
                            continue;
                        }

                        writeln!(file, "#### {}\n", benchmark)?;

                        // Get the latest result
                        let latest = results.iter().max_by_key(|r| r.timestamp).unwrap();

                        writeln!(file, "- **Type**: {}", latest.benchmark_type)?;
                        writeln!(file, "- **Unit**: {}", latest.unit)?;
                        writeln!(file, "- **Latest Mean**: {:.2}", latest.mean)?;
                        writeln!(file, "- **Latest Median**: {:.2}", latest.median)?;
                        writeln!(file, "- **Latest Min**: {:.2}", latest.min)?;
                        writeln!(file, "- **Latest Max**: {:.2}", latest.max)?;
                        if let Some(throughput) = latest.throughput {
                            writeln!(file, "- **Latest Throughput**: {:.2} ops/sec", throughput)?;
                        }
                        writeln!(file, "- **Latest Run**: {}", latest.timestamp.to_rfc3339())?;

                        // Generate chart
                        if results.len() > 1 {
                            let chart_path = self.generate_chart(component, benchmark)?;
                            let relative_path = chart_path
                                .strip_prefix(&self.output_dir)
                                .unwrap_or(&chart_path);
                            writeln!(
                                file,
                                "\n![{} Benchmark Chart]({})\n",
                                benchmark,
                                relative_path.display()
                            )?;
                        }
                    }
                }
            }
        }

        Ok(report_path)
    }

    /// Generate a chart for a benchmark
    fn generate_chart(&self, component: &str, benchmark: &str) -> io::Result<PathBuf> {
        if let Some(results) = self.metrics.get_benchmark_results(component, benchmark) {
            if results.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Not enough data points for chart",
                ));
            }

            let charts_dir = self.output_dir.join("charts");
            fs::create_dir_all(&charts_dir)?;

            let chart_path = charts_dir.join(format!("{}_{}.png", component, benchmark));

            // Sort results by timestamp
            let mut sorted_results = results.clone();
            sorted_results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // Extract data points
            let timestamps: Vec<DateTime<Utc>> =
                sorted_results.iter().map(|r| r.timestamp).collect();
            let means: Vec<f64> = sorted_results.iter().map(|r| r.mean).collect();

            // Determine y-axis range
            let min_y = means.iter().fold(f64::INFINITY, |a, &b| a.min(b)) * 0.9;
            let max_y = means.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) * 1.1;

            // Create the chart
            let root = BitMapBackend::new(&chart_path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)?;

            let mut chart = ChartBuilder::on(&root)
                .caption(format!("{} - {}", component, benchmark), ("sans-serif", 30))
                .margin(10)
                .x_label_area_size(40)
                .y_label_area_size(60)
                .build_cartesian_2d(
                    timestamps[0]..timestamps[timestamps.len() - 1],
                    min_y..max_y,
                )?;

            chart
                .configure_mesh()
                .x_labels(5)
                .y_labels(10)
                .x_label_formatter(&|x| x.format("%Y-%m-%d").to_string())
                .y_label_formatter(&|y| format!("{:.2}", y))
                .draw()?;

            // Draw the line
            chart.draw_series(LineSeries::new(
                timestamps.iter().zip(means.iter()).map(|(x, y)| (*x, *y)),
                &RED,
            ))?;

            // Draw points
            chart.draw_series(PointSeries::of_element(
                timestamps.iter().zip(means.iter()).map(|(x, y)| (*x, *y)),
                5,
                &RED,
                &|c, s, st| {
                    return EmptyElement::at(c) + Circle::new((0, 0), s, st.filled());
                },
            ))?;

            root.present()?;

            return Ok(chart_path);
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Benchmark results not found",
        ))
    }

    /// Generate a regression report
    pub fn generate_regression_report(&self, threshold: f64) -> io::Result<PathBuf> {
        let regressions = self.metrics.detect_regressions(threshold);
        let report_path = self.output_dir.join("regression_report.md");
        let mut file = File::create(&report_path)?;

        writeln!(file, "# IntelliRouter Performance Regression Report")?;
        writeln!(file, "\nGenerated at: {}\n", Utc::now().to_rfc3339())?;
        writeln!(file, "Regression threshold: {:.1}%\n", threshold * 100.0)?;

        if regressions.is_empty() {
            writeln!(file, "No performance regressions detected.")?;
        } else {
            writeln!(file, "## Detected Regressions\n")?;
            writeln!(
                file,
                "| Component | Benchmark | Previous | Current | Change | Type |"
            )?;
            writeln!(
                file,
                "|-----------|-----------|----------|---------|--------|------|"
            )?;

            for regression in &regressions {
                writeln!(
                    file,
                    "| {} | {} | {:.2} | {:.2} | {:.2}% | {:?} |",
                    regression.component,
                    regression.benchmark,
                    regression.previous_value,
                    regression.current_value,
                    regression.change_percentage,
                    regression.regression_type
                )?;
            }

            writeln!(file, "\n## Regression Details\n")?;

            for (i, regression) in regressions.iter().enumerate() {
                writeln!(
                    file,
                    "### Regression {}: {} - {}\n",
                    i + 1,
                    regression.component,
                    regression.benchmark
                )?;
                writeln!(file, "- **Type**: {:?}", regression.regression_type)?;
                writeln!(
                    file,
                    "- **Previous Value**: {:.2}",
                    regression.previous_value
                )?;
                writeln!(file, "- **Current Value**: {:.2}", regression.current_value)?;
                writeln!(file, "- **Change**: {:.2}%", regression.change_percentage)?;
                writeln!(
                    file,
                    "- **Previous Run**: {}",
                    regression.previous_timestamp.to_rfc3339()
                )?;
                writeln!(
                    file,
                    "- **Current Run**: {}",
                    regression.current_timestamp.to_rfc3339()
                )?;

                // Generate chart for this regression
                if let Some(results) = self
                    .metrics
                    .get_benchmark_results(&regression.component, &regression.benchmark)
                {
                    if results.len() > 1 {
                        let chart_path =
                            self.generate_chart(&regression.component, &regression.benchmark)?;
                        let relative_path = chart_path
                            .strip_prefix(&self.output_dir)
                            .unwrap_or(&chart_path);
                        writeln!(
                            file,
                            "\n![{} Benchmark Chart]({})\n",
                            regression.benchmark,
                            relative_path.display()
                        )?;
                    }
                }
            }
        }

        Ok(report_path)
    }

    /// Generate a CI report
    pub fn generate_ci_report(&self, threshold: f64) -> io::Result<(PathBuf, bool)> {
        let regressions = self.metrics.detect_regressions(threshold);
        let report_path = self.output_dir.join("ci_report.md");
        let mut file = File::create(&report_path)?;

        writeln!(file, "# IntelliRouter CI Performance Report")?;
        writeln!(file, "\nGenerated at: {}\n", Utc::now().to_rfc3339())?;

        let has_regressions = !regressions.is_empty();

        if has_regressions {
            writeln!(file, "## ⚠️ Performance Regressions Detected\n")?;
            writeln!(
                file,
                "| Component | Benchmark | Previous | Current | Change | Type |"
            )?;
            writeln!(
                file,
                "|-----------|-----------|----------|---------|--------|------|"
            )?;

            for regression in &regressions {
                writeln!(
                    file,
                    "| {} | {} | {:.2} | {:.2} | {:.2}% | {:?} |",
                    regression.component,
                    regression.benchmark,
                    regression.previous_value,
                    regression.current_value,
                    regression.change_percentage,
                    regression.regression_type
                )?;
            }

            writeln!(file, "\nPerformance regressions exceeding the threshold of {:.1}% were detected. Please review the changes and ensure they don't negatively impact performance.", threshold * 100.0)?;
        } else {
            writeln!(file, "## ✅ No Performance Regressions\n")?;
            writeln!(
                file,
                "No performance regressions exceeding the threshold of {:.1}% were detected.",
                threshold * 100.0
            )?;
        }

        // Summary of benchmarks
        writeln!(file, "\n## Benchmark Summary\n")?;

        let components = self.metrics.get_components();
        for component in &components {
            if let Some(benchmarks) = self.metrics.get_benchmarks(component) {
                writeln!(file, "### {}\n", component)?;

                for benchmark in benchmarks {
                    if let Some(results) = self.metrics.get_benchmark_results(component, benchmark)
                    {
                        if results.is_empty() {
                            continue;
                        }

                        // Get the latest result
                        let latest = results.iter().max_by_key(|r| r.timestamp).unwrap();

                        writeln!(
                            file,
                            "- **{}**: {:.2} {} (mean)",
                            benchmark, latest.mean, latest.unit
                        )?;
                    }
                }

                writeln!(file)?;
            }
        }

        Ok((report_path, has_regressions))
    }
}
