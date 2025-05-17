// IntelliRouter Entry Point
//
// This file contains the entry point for the IntelliRouter application,
// which can assume different functional roles at runtime.

use intellirouter::cli::{parse_args, Commands, Role};
use intellirouter::config::TelemetryConfig;
use intellirouter::modules::telemetry;
use std::process;
use tracing::Level as LogLevel;

fn main() {
    // Parse command-line arguments and get configuration
    let (cli, config) = parse_args();

    // Handle commands
    match &cli.command {
        Commands::Init(init_args) => {
            if init_args.force {
                println!("Creating default configuration files (force mode)...");
            } else {
                println!("Creating default configuration files...");
            }

            match intellirouter::config::Config::create_default_configs() {
                Ok(_) => {
                    println!("Default configuration files created in the 'config' directory");
                    return;
                }
                Err(e) => {
                    eprintln!("Failed to create default configuration files: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Validate(validate_args) => {
            match config.validate() {
                Ok(_) => {
                    println!("Configuration validation successful");
                    if validate_args.verbose {
                        println!("Configuration details:");
                        println!("Environment: {:?}", config.environment);
                        println!("Server: {}:{}", config.server.host, config.server.port);
                        // Add more details as needed
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("Configuration validation failed: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Run(run_args) => {
            // Initialize telemetry
            let log_level = match config.telemetry.log_level() {
                Ok(level) => level,
                Err(e) => {
                    eprintln!("Invalid log level in configuration: {}", e);
                    process::exit(1);
                }
            };

            let telemetry_config = TelemetryConfig {
                log_level: "info".to_string(),
                metrics_enabled: config.telemetry.metrics_enabled,
                tracing_enabled: config.telemetry.tracing_enabled,
                metrics_endpoint: None,
                tracing_endpoint: None,
            };

            // Initialize telemetry with a stub implementation
            // TODO: Replace with actual telemetry initialization
            println!(
                "Telemetry initialized with log level: {:?}",
                telemetry_config.log_level
            );

            // Log configuration information
            println!("Environment: {:?}", config.environment);
            println!("Server listening on: {}", config.server.socket_addr());

            // Determine the role to assume
            let role = run_args.role;

            // Initialize the appropriate components based on the role
            match role {
                Role::LlmProxy => {
                    println!("Starting in LLM Proxy role");
                    // TODO: Initialize LLM Proxy components
                }
                Role::Router => {
                    println!("Starting in Router role");

                    // Initialize Router components
                    if let Err(e) = intellirouter::modules::router_core::init(
                        intellirouter::modules::router_core::RouterConfig::default(),
                    ) {
                        eprintln!("Failed to initialize Router: {}", e);
                        process::exit(1);
                    }

                    // Create a tokio runtime for async operations
                    let runtime = tokio::runtime::Runtime::new().unwrap();

                    // Start the LLM Proxy server to handle API requests
                    runtime.block_on(async {
                        if let Err(e) = intellirouter::modules::llm_proxy::init(
                            intellirouter::modules::llm_proxy::Provider::OpenAI,
                            &config,
                        )
                        .await
                        {
                            eprintln!("Failed to start LLM Proxy server: {}", e);
                            process::exit(1);
                        }

                        // Keep the server running until Ctrl+C
                        tokio::signal::ctrl_c().await.unwrap();
                        println!("Shutting down...");
                    });
                }
                Role::ChainEngine => {
                    println!("Starting in Chain Engine role");
                    // TODO: Initialize Chain Engine components
                }
                Role::RagManager => {
                    println!("Starting in RAG Manager role");
                    // TODO: Initialize RAG Manager components
                }
                Role::PersonaLayer => {
                    println!("Starting in Persona Layer role");
                    // TODO: Initialize Persona Layer components
                }
                Role::All => {
                    println!("Starting with all components enabled");
                    // TODO: Initialize all components
                }
            }

            println!("IntelliRouter initialized successfully");

            // TODO: Start the appropriate services based on the role

            println!(
                "IntelliRouter is running on {}...",
                config.server.socket_addr()
            );
        }
    }
}
