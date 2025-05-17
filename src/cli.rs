// CLI Argument Parsing
//
// This module handles command-line argument parsing for the IntelliRouter application,
// which can assume different functional roles at runtime.

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use std::net::IpAddr;
use std::str::FromStr;

use crate::config::{AppEnvironment, Config};

/// Available roles that IntelliRouter can assume
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Role {
    /// LLM Proxy role - Provides an OpenAI-compatible API
    LlmProxy,
    /// Router role - Routes requests to appropriate LLM backends
    Router,
    /// Chain Engine role - Orchestrates multi-step LLM workflows
    ChainEngine,
    /// RAG Manager role - Manages retrieval-augmented generation
    RagManager,
    /// Persona Layer role - Manages system prompts and personas
    PersonaLayer,
    /// Audit Controller role - Orchestrates testing and validation
    Audit,
    /// All roles combined
    All,
}

/// Command line arguments for IntelliRouter
#[derive(Parser, Debug)]
#[command(
    name = "intellirouter",
    author = "IntelliRouter Team",
    version,
    about = "A flexible LLM orchestration system with intelligent routing between multiple LLM backends",
    long_about = None
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Environment (development, testing, production)
    #[arg(short, long, global = true)]
    pub environment: Option<String>,

    /// Log level (debug, info, warning, error)
    #[arg(short, long, global = true)]
    pub log_level: Option<String>,
}

/// Subcommands for IntelliRouter
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run IntelliRouter in a specific role
    Run(RunArgs),

    /// Initialize configuration files
    Init(InitArgs),

    /// Validate configuration
    Validate(ValidateArgs),
}

/// Arguments for the run command
#[derive(ClapArgs, Debug)]
pub struct RunArgs {
    /// Role to assume
    #[arg(short, long, value_enum, default_value_t = Role::All)]
    pub role: Role,

    /// Host address to bind to
    #[arg(long)]
    pub host: Option<IpAddr>,

    /// Port to listen on
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Maximum number of concurrent connections
    #[arg(long)]
    pub max_connections: Option<usize>,

    /// Request timeout in seconds
    #[arg(long)]
    pub request_timeout: Option<u64>,

    /// Enable CORS
    #[arg(long)]
    pub cors_enabled: Option<bool>,

    /// CORS allowed origins (comma-separated)
    #[arg(long)]
    pub cors_allowed_origins: Option<Vec<String>>,

    /// Memory backend type (memory, redis, file)
    #[arg(long)]
    pub memory_backend: Option<String>,

    /// Redis URL for memory backend
    #[arg(long)]
    pub redis_url: Option<String>,

    /// File path for memory backend
    #[arg(long)]
    pub file_path: Option<String>,

    /// Default LLM provider
    #[arg(long)]
    pub default_provider: Option<String>,

    /// Enable authentication
    #[arg(long)]
    pub auth_enabled: Option<bool>,

    /// Authentication method (jwt, api_key)
    #[arg(long)]
    pub auth_method: Option<String>,

    /// Enable RAG
    #[arg(long)]
    pub rag_enabled: Option<bool>,

    /// Vector database URL for RAG
    #[arg(long)]
    pub vector_db_url: Option<String>,

    /// Enable persona layer
    #[arg(long)]
    pub persona_enabled: Option<bool>,

    /// Default persona
    #[arg(long)]
    pub default_persona: Option<String>,

    /// Enable plugins
    #[arg(long)]
    pub plugins_enabled: Option<bool>,
}

/// Arguments for the init command
#[derive(ClapArgs, Debug)]
pub struct InitArgs {
    /// Force overwrite of existing configuration files
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the validate command
#[derive(ClapArgs, Debug)]
pub struct ValidateArgs {
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Parse command-line arguments and return a Config
pub fn parse_args() -> (Cli, Config) {
    let cli = Cli::parse();

    // Load base configuration
    let mut config = if let Some(config_path) = &cli.config {
        // Load from specified file
        match Config::from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load configuration from file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Load from all sources with proper precedence
        match Config::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load configuration: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Override with command-line arguments
    if let Some(env_str) = &cli.environment {
        match AppEnvironment::from_str(env_str) {
            Ok(env) => config.environment = env,
            Err(e) => {
                eprintln!("Invalid environment: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(log_level) = &cli.log_level {
        config.telemetry.log_level = log_level.clone();
    }

    // Apply role-specific overrides if using the run command
    if let Commands::Run(run_args) = &cli.command {
        apply_run_args_to_config(&mut config, run_args);
    }

    (cli, config)
}

/// Apply run command arguments to the configuration
fn apply_run_args_to_config(config: &mut Config, args: &RunArgs) {
    // Server configuration
    if let Some(host) = args.host {
        config.server.host = host;
    }

    if let Some(port) = args.port {
        config.server.port = port;
    }

    if let Some(max_connections) = args.max_connections {
        config.server.max_connections = max_connections;
    }

    if let Some(request_timeout) = args.request_timeout {
        config.server.request_timeout_secs = request_timeout;
    }

    if let Some(cors_enabled) = args.cors_enabled {
        config.server.cors_enabled = cors_enabled;
    }

    if let Some(cors_allowed_origins) = &args.cors_allowed_origins {
        config.server.cors_allowed_origins = cors_allowed_origins.clone();
    }

    // Memory configuration
    if let Some(memory_backend) = &args.memory_backend {
        config.memory.backend_type = memory_backend.clone();
    }

    if let Some(redis_url) = &args.redis_url {
        config.memory.redis_url = Some(redis_url.clone());
    }

    if let Some(file_path) = &args.file_path {
        config.memory.file_path = Some(file_path.clone());
    }

    // Model registry configuration
    if let Some(default_provider) = &args.default_provider {
        config.model_registry.default_provider = default_provider.clone();
    }

    // Auth configuration
    if let Some(auth_enabled) = args.auth_enabled {
        config.auth.auth_enabled = auth_enabled;
    }

    if let Some(auth_method) = &args.auth_method {
        config.auth.auth_method = auth_method.clone();
    }

    // RAG configuration
    if let Some(rag_enabled) = args.rag_enabled {
        config.rag.enabled = rag_enabled;
    }

    if let Some(vector_db_url) = &args.vector_db_url {
        config.rag.vector_db_url = Some(vector_db_url.clone());
    }

    // Persona layer configuration
    if let Some(persona_enabled) = args.persona_enabled {
        config.persona_layer.enabled = persona_enabled;
    }

    if let Some(default_persona) = &args.default_persona {
        config.persona_layer.default_persona = default_persona.clone();
    }

    // Plugin SDK configuration
    if let Some(plugins_enabled) = args.plugins_enabled {
        config.plugin_sdk.enabled = plugins_enabled;
    }
}
