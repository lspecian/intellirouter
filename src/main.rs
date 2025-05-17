// IntelliRouter Entry Point
//
// This file contains the entry point for the IntelliRouter application,
// which can assume different functional roles at runtime.

use intellirouter::cli::{parse_args, Commands, Role};
use intellirouter::modules::telemetry::{self, LogLevel};
use std::process;

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

            let telemetry_config = telemetry::TelemetryConfig {
                log_level,
                metrics_enabled: config.telemetry.metrics_enabled,
                tracing_enabled: config.telemetry.tracing_enabled,
            };

            if let Err(e) = telemetry::init(telemetry_config) {
                eprintln!("Failed to initialize telemetry: {}", e);
                process::exit(1);
            }

            // Log configuration information
            telemetry::log(
                LogLevel::Info,
                &format!("Environment: {:?}", config.environment),
            );
            telemetry::log(
                LogLevel::Info,
                &format!("Server listening on: {}", config.server.socket_addr()),
            );

            // Determine the role to assume
            let role = run_args.role;

            // Initialize the appropriate components based on the role
            match role {
                Role::LlmProxy => {
                    telemetry::log(LogLevel::Info, "Starting in LLM Proxy role");
                    // TODO: Initialize LLM Proxy components
                }
                Role::Router => {
                    telemetry::log(LogLevel::Info, "Starting in Router role");
                    // TODO: Initialize Router components
                }
                Role::ChainEngine => {
                    telemetry::log(LogLevel::Info, "Starting in Chain Engine role");
                    // TODO: Initialize Chain Engine components
                }
                Role::RagManager => {
                    telemetry::log(LogLevel::Info, "Starting in RAG Manager role");
                    // TODO: Initialize RAG Manager components
                }
                Role::PersonaLayer => {
                    telemetry::log(LogLevel::Info, "Starting in Persona Layer role");
                    // TODO: Initialize Persona Layer components
                }
                Role::All => {
                    telemetry::log(LogLevel::Info, "Starting with all components enabled");
                    // TODO: Initialize all components
                }
            }

            telemetry::log(LogLevel::Info, "IntelliRouter initialized successfully");

            // TODO: Start the appropriate services based on the role

            println!(
                "IntelliRouter is running on {}...",
                config.server.socket_addr()
            );
        }
    }
}
