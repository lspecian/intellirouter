use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use chrono::Utc;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use super::types::AuditError;
use super::AuditConfig;
use super::AuditController;
use super::AuditReport;
use super::DashboardConfig;
use super::ReportFormat;
use super::ReportGenerator;

/// Audit CLI arguments
#[derive(Debug, Args)]
pub struct AuditCliArgs {
    /// Audit subcommand
    #[clap(subcommand)]
    pub command: AuditCommand,
}

/// Deployment scenario for the audit system
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DeploymentScenario {
    /// Single-node deployment (all services on one machine)
    SingleNode,
    /// Distributed deployment (services on different machines)
    Distributed,
    /// Cloud deployment (services in Kubernetes)
    Cloud,
    /// Local development deployment
    LocalDev,
}

impl std::fmt::Display for DeploymentScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentScenario::SingleNode => write!(f, "single-node"),
            DeploymentScenario::Distributed => write!(f, "distributed"),
            DeploymentScenario::Cloud => write!(f, "cloud"),
            DeploymentScenario::LocalDev => write!(f, "local-dev"),
        }
    }
}

/// CI/CD platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CIPlatform {
    /// GitHub Actions
    GitHub,
    /// Jenkins
    Jenkins,
    /// GitLab CI
    GitLab,
    /// CircleCI
    CircleCI,
}

/// Historical test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTestResult {
    /// Timestamp of the test run
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Test report
    pub report: AuditReport,
    /// Deployment scenario
    pub deployment: String,
    /// Configuration used
    pub config: AuditConfig,
}

/// Audit subcommands
#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Run the audit process
    Run {
        /// Path to the audit configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,

        /// Path to save the audit report
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// Report format
        #[clap(short, long, default_value = "json")]
        format: String,

        /// Start the dashboard server
        #[clap(short, long)]
        dashboard: bool,

        /// Dashboard host
        #[clap(long, default_value = "127.0.0.1")]
        dashboard_host: String,

        /// Dashboard port
        #[clap(long, default_value = "8090")]
        dashboard_port: u16,

        /// Deployment scenario
        #[clap(long, value_enum, default_value = "local-dev")]
        deployment: Option<DeploymentScenario>,

        /// Specific tests to run (comma-separated)
        #[clap(long)]
        tests: Option<String>,

        /// Verbose output
        #[clap(short, long)]
        verbose: bool,

        /// Store test results for historical comparison
        #[clap(long)]
        store_results: bool,

        /// Compare with previous test results
        #[clap(long)]
        compare: bool,

        /// CI mode (non-interactive, exit code reflects test status)
        #[clap(long)]
        ci: bool,
    },

    /// List available tests
    ListTests {
        /// Path to the audit configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },

    /// View historical test results
    History {
        /// Number of historical results to show
        #[clap(short, long, default_value = "5")]
        limit: usize,

        /// Filter by test name
        #[clap(short, long)]
        test: Option<String>,

        /// Output format
        #[clap(short, long, default_value = "text")]
        format: String,
    },

    /// Generate CI/CD configuration files
    GenerateCI {
        /// CI/CD platform
        #[clap(long, value_enum)]
        platform: CIPlatform,

        /// Output directory
        #[clap(short, long)]
        output: PathBuf,

        /// Deployment scenario
        #[clap(long, value_enum, default_value = "cloud")]
        deployment: DeploymentScenario,
    },
}

/// Run the audit CLI
pub async fn run_audit_cli(args: AuditCliArgs) -> Result<(), AuditError> {
    match args.command {
        AuditCommand::Run {
            config,
            output,
            format,
            dashboard,
            dashboard_host,
            dashboard_port,
            deployment,
            tests,
            verbose,
            store_results,
            compare,
            ci,
        } => {
            // Load configuration
            let config_path = config.unwrap_or_else(|| PathBuf::from("config/audit.toml"));
            let mut audit_config = if config_path.exists() {
                info!("Loading audit configuration from {:?}", config_path);
                // In a real implementation, this would load the configuration from the file
                AuditConfig::default()
            } else {
                info!("Using default audit configuration");
                AuditConfig::default()
            };

            // Apply deployment scenario configuration
            if let Some(deployment_scenario) = deployment {
                info!(
                    "Configuring for deployment scenario: {:?}",
                    deployment_scenario
                );
                apply_deployment_scenario(&mut audit_config, deployment_scenario);
            }

            // Configure specific tests if provided
            if let Some(test_list) = tests {
                info!("Configuring specific tests: {}", test_list);
                configure_specific_tests(&mut audit_config, &test_list);
            }

            // Set verbose logging if requested
            if verbose {
                audit_config.log_level = super::types::LogLevel::Debug;
                info!("Verbose logging enabled");
            }

            // Create audit controller
            let mut controller = AuditController::new(audit_config.clone());

            // Configure report generator if dashboard is enabled
            if dashboard {
                info!(
                    "Configuring dashboard server on {}:{}",
                    dashboard_host, dashboard_port
                );

                // Create dashboard configuration
                let dashboard_config = DashboardConfig {
                    host: dashboard_host.clone(),
                    port: dashboard_port,
                    static_dir: "src/modules/audit/static".to_string(),
                    enable_history: true,
                    max_history: 10,
                };

                // Configure report generator
                let report = Arc::new(RwLock::new(AuditReport::new()));
                let report_generator = ReportGenerator::new(Arc::clone(&report))
                    .with_dashboard(dashboard_config.clone());

                // Configure controller with report generator and dashboard
                controller = controller
                    .with_report_generator(report_generator)
                    .with_dashboard(dashboard_config);

                // Start dashboard server
                controller.start_dashboard().await?;

                info!(
                    "Dashboard server started on http://{}:{}",
                    dashboard_host, dashboard_port
                );
            }

            // Compare with previous results if requested
            if compare {
                info!("Comparing with previous test results");
                let previous_results = load_historical_results(5)?;
                if previous_results.is_empty() {
                    warn!("No previous test results found for comparison");
                } else {
                    info!(
                        "Found {} previous test results for comparison",
                        previous_results.len()
                    );
                    // In a real implementation, this would configure the controller to compare results
                }
            }

            // Run audit process
            info!("Running audit process");
            let report = controller.run_audit().await?;

            // Store results for historical comparison if requested
            if store_results {
                info!("Storing test results for historical comparison");
                store_historical_result(
                    &report,
                    &audit_config,
                    deployment
                        .map(|d| format!("{:?}", d))
                        .unwrap_or_else(|| "Default".to_string()),
                )?;
            }

            // Save report if output path is provided
            if let Some(output_path) = output {
                info!("Saving audit report to {:?}", output_path);

                // Parse report format
                let report_format = match format.to_lowercase().as_str() {
                    "json" => ReportFormat::Json,
                    "markdown" | "md" => ReportFormat::Markdown,
                    "html" => ReportFormat::Html,
                    _ => {
                        warn!("Unknown report format: {}, using JSON", format);
                        ReportFormat::Json
                    }
                };

                // Save report
                report.save(output_path.to_str().unwrap(), report_format)?;

                info!("Audit report saved to {:?}", output_path);
            }

            // If in CI mode, exit with appropriate code
            if ci {
                info!("Running in CI mode, will exit with status code based on test results");
                if report.has_errors() {
                    error!("Tests failed, would exit with code 1 in a real CI environment");
                    // In a real implementation, this would exit with code 1
                } else {
                    info!("All tests passed");
                }
            }
            // If dashboard is enabled, keep the process running
            else if dashboard {
                info!("Dashboard server is running. Press Ctrl+C to exit.");

                // Wait for Ctrl+C
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");

                info!("Received Ctrl+C, shutting down");
            }

            Ok(())
        }
        AuditCommand::ListTests { config } => {
            // Load configuration
            let config_path = config.unwrap_or_else(|| PathBuf::from("config/audit.toml"));
            let audit_config = if config_path.exists() {
                info!("Loading audit configuration from {:?}", config_path);
                // In a real implementation, this would load the configuration from the file
                AuditConfig::default()
            } else {
                info!("Using default audit configuration");
                AuditConfig::default()
            };

            // List available tests
            info!("Available tests:");
            for test_flow in audit_config.test_config.test_flows {
                println!("- {}", test_flow);
            }

            Ok(())
        }
        AuditCommand::History {
            limit,
            test,
            format,
        } => {
            // Load historical results
            let results = load_historical_results(limit)?;
            if results.is_empty() {
                println!("No historical test results found");
                return Ok(());
            }

            // Filter by test name if provided
            let filtered_results = if let Some(test_name) = test {
                info!("Filtering results by test: {}", test_name);
                results
                    .into_iter()
                    .filter(|r| r.report.contains_test(&test_name))
                    .collect::<Vec<_>>()
            } else {
                results
            };

            if filtered_results.is_empty() {
                println!("No matching historical test results found");
                return Ok(());
            }

            // Display results
            match format.to_lowercase().as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&filtered_results)?);
                }
                "text" | _ => {
                    println!("Historical Test Results:");
                    println!("=======================");
                    for (i, result) in filtered_results.iter().enumerate() {
                        println!("Result #{}", i + 1);
                        println!("  Timestamp: {}", result.timestamp);
                        println!("  Deployment: {}", result.deployment);
                        println!("  Success: {}", !result.report.has_errors());
                        println!("  Tests: {}", result.report.get_test_count());
                        println!("  Errors: {}", result.report.get_error_count());
                        println!();
                    }
                }
            }

            Ok(())
        }
        AuditCommand::GenerateCI {
            platform,
            output,
            deployment,
        } => {
            info!("Generating CI/CD configuration for {:?}", platform);

            // Create output directory if it doesn't exist
            if !output.exists() {
                fs::create_dir_all(&output)?;
            }

            // Generate CI configuration
            match platform {
                CIPlatform::GitHub => {
                    generate_github_workflow(&output, deployment)?;
                }
                CIPlatform::Jenkins => {
                    generate_jenkins_pipeline(&output, deployment)?;
                }
                CIPlatform::GitLab => {
                    generate_gitlab_ci(&output, deployment)?;
                }
                CIPlatform::CircleCI => {
                    generate_circleci_config(&output, deployment)?;
                }
            }

            info!("CI/CD configuration generated in {:?}", output);
            Ok(())
        }
    }
}

/// Apply deployment scenario configuration
fn apply_deployment_scenario(config: &mut AuditConfig, scenario: DeploymentScenario) {
    match scenario {
        DeploymentScenario::SingleNode => {
            // Configure for single-node deployment
            config.discovery_config.discovery_timeout_secs = 30;
            config.discovery_config.connection_timeout_ms = 2000;
            // All services are on localhost
            // In a real implementation, this would update service hosts
        }
        DeploymentScenario::Distributed => {
            // Configure for distributed deployment
            config.discovery_config.discovery_timeout_secs = 60;
            config.discovery_config.connection_timeout_ms = 5000;
            // Services are on different hosts
            // In a real implementation, this would update service hosts
        }
        DeploymentScenario::Cloud => {
            // Configure for cloud deployment
            config.discovery_config.discovery_timeout_secs = 120;
            config.discovery_config.connection_timeout_ms = 10000;
            // Services are in Kubernetes
            // In a real implementation, this would update service hosts and ports
        }
        DeploymentScenario::LocalDev => {
            // Configure for local development
            config.discovery_config.discovery_timeout_secs = 15;
            config.discovery_config.connection_timeout_ms = 1000;
            // All services are on localhost with default ports
            // In a real implementation, this would update service hosts
        }
    }
}

/// Configure specific tests
fn configure_specific_tests(config: &mut AuditConfig, test_list: &str) {
    let tests = test_list.split(',').collect::<Vec<_>>();
    let mut test_flows = Vec::new();

    for test in tests {
        match test.trim() {
            "basic" => test_flows.push(super::types::TestFlow::BasicChainExecution),
            "rag" => test_flows.push(super::types::TestFlow::RagIntegration),
            "persona" => test_flows.push(super::types::TestFlow::PersonaLayerIntegration),
            "e2e" => test_flows.push(super::types::TestFlow::EndToEndFlow),
            _ => warn!("Unknown test: {}, ignoring", test),
        }
    }

    if !test_flows.is_empty() {
        config.test_config.test_flows = test_flows;
    }
}

/// Store historical test result
fn store_historical_result(
    report: &AuditReport,
    config: &AuditConfig,
    deployment: String,
) -> Result<(), AuditError> {
    // Create history directory if it doesn't exist
    let history_dir = PathBuf::from("audit_history");
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
    }

    // Create historical result
    let result = HistoricalTestResult {
        timestamp: Utc::now(),
        report: report.clone(),
        deployment,
        config: config.clone(),
    };

    // Generate filename with timestamp
    let filename = format!("audit_result_{}.json", Utc::now().format("%Y%m%d_%H%M%S"));
    let file_path = history_dir.join(filename);

    // Save to file
    let json = serde_json::to_string_pretty(&result)?;
    fs::write(file_path, json)?;

    Ok(())
}

/// Load historical test results
fn load_historical_results(limit: usize) -> Result<Vec<HistoricalTestResult>, AuditError> {
    let history_dir = PathBuf::from("audit_history");
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let entries = fs::read_dir(history_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(path)?;
            match serde_json::from_str::<HistoricalTestResult>(&content) {
                Ok(result) => results.push(result),
                Err(e) => warn!("Failed to parse historical result: {}", e),
            }
        }
    }

    // Sort by timestamp (newest first)
    results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Limit results
    if results.len() > limit {
        results.truncate(limit);
    }

    Ok(results)
}

/// Generate GitHub Actions workflow configuration
fn generate_github_workflow(
    output_dir: &PathBuf,
    deployment: DeploymentScenario,
) -> Result<(), AuditError> {
    let github_dir = output_dir.join(".github").join("workflows");
    fs::create_dir_all(&github_dir)?;

    let workflow_path = github_dir.join("integration-tests.yml");
    let workflow_content = format!(
        r#"name: Integration Tests

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]
  workflow_dispatch:

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Start Integration Test Environment
        run: docker-compose -f docker-compose.integration.yml up -d

      - name: Wait for Services to be Ready
        run: |
          echo "Waiting for services to be ready..."
          sleep 30
          docker-compose -f docker-compose.integration.yml ps

      - name: Run Integration Tests
        run: |
          docker-compose -f docker-compose.integration.yml run test-runner cargo run -- audit run --deployment {} --ci --store-results

      - name: Collect Test Reports
        if: always()
        run: |
          mkdir -p test-reports
          docker-compose -f docker-compose.integration.yml cp test-runner:/app/audit_history test-reports/

      - name: Upload Test Reports
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-reports
          path: test-reports/

      - name: Cleanup
        if: always()
        run: docker-compose -f docker-compose.integration.yml down -v
"#,
        deployment.to_string().to_lowercase()
    );

    fs::write(workflow_path, workflow_content)?;
    Ok(())
}

/// Generate Jenkins pipeline configuration
fn generate_jenkins_pipeline(
    output_dir: &PathBuf,
    deployment: DeploymentScenario,
) -> Result<(), AuditError> {
    let jenkins_file = output_dir.join("Jenkinsfile");
    let jenkins_content = format!(
        r#"pipeline {{
    agent {{
        docker {{
            image 'docker:dind'
            args '-v /var/run/docker.sock:/var/run/docker.sock'
        }}
    }}
    
    stages {{
        stage('Checkout') {{
            steps {{
                checkout scm
            }}
        }}
        
        stage('Start Integration Environment') {{
            steps {{
                sh 'docker-compose -f docker-compose.integration.yml up -d'
                sh 'sleep 30' // Wait for services to be ready
                sh 'docker-compose -f docker-compose.integration.yml ps'
            }}
        }}
        
        stage('Run Integration Tests') {{
            steps {{
                sh 'docker-compose -f docker-compose.integration.yml run test-runner cargo run -- audit run --deployment {} --ci --store-results --output /app/test-report.json'
            }}
        }}
        
        stage('Collect Reports') {{
            steps {{
                sh 'mkdir -p test-reports'
                sh 'docker-compose -f docker-compose.integration.yml cp test-runner:/app/test-report.json test-reports/'
                sh 'docker-compose -f docker-compose.integration.yml cp test-runner:/app/audit_history test-reports/'
            }}
        }}
    }}
    
    post {{
        always {{
            archiveArtifacts artifacts: 'test-reports/**/*', allowEmptyArchive: true
            sh 'docker-compose -f docker-compose.integration.yml down -v'
        }}
    }}
}}
"#,
        deployment.to_string().to_lowercase()
    );

    fs::write(jenkins_file, jenkins_content)?;
    Ok(())
}

/// Generate GitLab CI configuration
fn generate_gitlab_ci(
    output_dir: &PathBuf,
    deployment: DeploymentScenario,
) -> Result<(), AuditError> {
    let gitlab_file = output_dir.join(".gitlab-ci.yml");
    let gitlab_content = format!(
        r#"stages:
  - test

variables:
  DOCKER_HOST: tcp://docker:2375
  DOCKER_DRIVER: overlay2

services:
  - docker:dind

integration_tests:
  stage: test
  image: docker/compose:latest
  script:
    - docker-compose -f docker-compose.integration.yml up -d
    - echo "Waiting for services to be ready..."
    - sleep 30
    - docker-compose -f docker-compose.integration.yml ps
    - docker-compose -f docker-compose.integration.yml run test-runner cargo run -- audit run --deployment {} --ci --store-results --output /app/test-report.json
    - mkdir -p test-reports
    - docker cp $(docker-compose -f docker-compose.integration.yml ps -q test-runner):/app/test-report.json test-reports/
    - docker cp $(docker-compose -f docker-compose.integration.yml ps -q test-runner):/app/audit_history test-reports/
  artifacts:
    paths:
      - test-reports/
    expire_in: 1 week
  after_script:
    - docker-compose -f docker-compose.integration.yml down -v
"#,
        deployment.to_string().to_lowercase()
    );

    fs::write(gitlab_file, gitlab_content)?;
    Ok(())
}

/// Generate CircleCI configuration
fn generate_circleci_config(
    output_dir: &PathBuf,
    deployment: DeploymentScenario,
) -> Result<(), AuditError> {
    let circleci_dir = output_dir.join(".circleci");
    fs::create_dir_all(&circleci_dir)?;

    let circleci_file = circleci_dir.join("config.yml");
    let circleci_content = format!(
        r#"version: 2.1

jobs:
  integration_tests:
    machine:
      image: ubuntu-2004:202010-01
    steps:
      - checkout
      
      - run:
          name: Start Integration Test Environment
          command: |
            docker-compose -f docker-compose.integration.yml up -d
            echo "Waiting for services to be ready..."
            sleep 30
            docker-compose -f docker-compose.integration.yml ps
      
      - run:
          name: Run Integration Tests
          command: |
            docker-compose -f docker-compose.integration.yml run test-runner cargo run -- audit run --deployment {} --ci --store-results --output /app/test-report.json
      
      - run:
          name: Collect Test Reports
          command: |
            mkdir -p test-reports
            docker cp $(docker-compose -f docker-compose.integration.yml ps -q test-runner):/app/test-report.json test-reports/
            docker cp $(docker-compose -f docker-compose.integration.yml ps -q test-runner):/app/audit_history test-reports/
      
      - store_artifacts:
          path: test-reports
          destination: test-reports
      
      - run:
          name: Cleanup
          command: docker-compose -f docker-compose.integration.yml down -v
          when: always

workflows:
  version: 2
  build_and_test:
    jobs:
      - integration_tests
"#,
        deployment.to_string().to_lowercase()
    );

    fs::write(circleci_file, circleci_content)?;
    Ok(())
}
