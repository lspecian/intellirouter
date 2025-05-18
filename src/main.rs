use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use intellirouter::config::Config;
// Import public interfaces only
use intellirouter::modules::chain_engine::ChainEngine;
use intellirouter::modules::memory::{InMemoryBackend, MemoryManager};
use intellirouter::modules::model_registry::api::ModelRegistryApi;
use intellirouter::modules::model_registry::storage::ModelRegistry;
use intellirouter::modules::persona_layer::manager::PersonaManager;
use intellirouter::modules::rag_manager::manager::RagManager;
use intellirouter::modules::router_core::router::RouterImpl;
use intellirouter::modules::telemetry::telemetry::TelemetryManager;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the IntelliRouter server
    Run {
        /// Role to run (router, orchestrator, rag-injector, summarizer, all)
        #[arg(short, long, default_value = "all")]
        role: Role,

        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Environment (development, production)
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// Generate a default configuration file
    GenerateConfig {
        /// Output file path
        #[arg(short, long, default_value = "config.toml")]
        output: PathBuf,

        /// Environment (development, production)
        #[arg(short, long, default_value = "development")]
        env: String,
    },
}

#[derive(Clone, Debug)]
enum Role {
    Router,
    Orchestrator,
    RagInjector,
    Summarizer,
    All,
    Audit,
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "router" => Ok(Role::Router),
            "orchestrator" => Ok(Role::Orchestrator),
            "rag-injector" | "raginjector" => Ok(Role::RagInjector),
            "summarizer" => Ok(Role::Summarizer),
            "all" => Ok(Role::All),
            "audit" => Ok(Role::Audit),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize telemetry
    // Set up basic logging
    TelemetryManager::setup_logging().expect("Failed to set up logging");

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { role, config, env } => {
            // Load configuration
            let config_path = config.unwrap_or_else(|| {
                let mut path = PathBuf::from("config");
                path.push(format!("{}.toml", env));
                path
            });

            println!("Loading configuration from {:?}", config_path);
            let config = Config::from_file(config_path.to_str().unwrap())
                .expect("Failed to load configuration");

            // Initialize telemetry with configuration
            let metrics_addr: SocketAddr = config.server.socket_addr();
            let telemetry = Arc::new(TelemetryManager::new(
                "intellirouter".to_string(),
                env.clone(),
                env!("CARGO_PKG_VERSION").to_string(),
            ));

            // Run the appropriate role
            match role {
                Role::Router => {
                    println!("Starting in Router role");

                    // Create model registry client
                    // Create model registry API
                    let model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create router
                    // Create a simple router config
                    let router_config =
                        intellirouter::modules::router_core::config::RouterConfig::default();

                    // Create model registry
                    let model_registry = Arc::new(ModelRegistry::new());

                    // Create router
                    let router = RouterImpl::new(router_config, model_registry)
                        .expect("Failed to create router");

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create chain engine
                    let chain_engine = ChainEngine::new();

                    // Create app with telemetry and LLM proxy routes
                    let app_state = intellirouter::modules::llm_proxy::server::AppState {
                        provider: intellirouter::modules::llm_proxy::Provider::OpenAI,
                        config:
                            intellirouter::modules::llm_proxy::server::ServerConfig::from_config(
                                &config,
                            ),
                        shared: Arc::new(tokio::sync::Mutex::new(
                            intellirouter::modules::llm_proxy::server::SharedState::new(),
                        )),
                        telemetry: Some(telemetry.clone()),
                        cost_calculator: Some(Arc::new(
                            intellirouter::modules::telemetry::CostCalculator::new(),
                        )),
                    };

                    // Create router with routes
                    let app = intellirouter::modules::llm_proxy::server::create_router(app_state);

                    // Start server
                    let addr = config.server.socket_addr();
                    println!("Router listening on {}", addr);

                    // Create TCP listener
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("Failed to bind to address");

                    println!("Health check endpoints available at:");
                    println!("  - /health");
                    println!("  - /readiness");
                    println!("  - /diagnostics");

                    // Start server
                    axum::serve(listener, app).await.expect("Server error");
                }
                Role::Orchestrator => {
                    println!("Starting in Orchestrator (Chain Engine) role");

                    // Create model registry client
                    // Create model registry API
                    let model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create rag manager
                    let rag_manager = RagManager::new();

                    // Create persona layer manager
                    let persona_manager = PersonaManager::new();

                    // Create chain engine
                    let chain_engine = ChainEngine::new();

                    // Create app with telemetry
                    let app = axum::Router::new().with_state(telemetry.clone());

                    // Start server
                    let addr = SocketAddr::new(config.server.host, config.server.port + 1);
                    println!("Chain Engine listening on {}", addr);

                    // Create TCP listener
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("Failed to bind to address");

                    println!("Health check endpoints available at:");
                    println!("  - /health");
                    println!("  - /readiness");
                    println!("  - /diagnostics");

                    // Start server
                    axum::serve(listener, app).await.expect("Server error");
                }
                Role::RagInjector => {
                    println!("Starting in RAG Injector (RAG Manager) role");

                    // Create model registry client
                    // Create model registry API
                    let model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create RAG manager
                    let rag_manager = RagManager::new();

                    // Create app with telemetry
                    let app = axum::Router::new().with_state(telemetry.clone());

                    // Start server
                    let addr = SocketAddr::new(config.server.host, config.server.port + 2);
                    println!("RAG Manager listening on {}", addr);

                    // Create TCP listener
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("Failed to bind to address");

                    println!("Health check endpoints available at:");
                    println!("  - /health");
                    println!("  - /readiness");
                    println!("  - /diagnostics");

                    // Start server
                    axum::serve(listener, app).await.expect("Server error");
                }
                Role::Summarizer => {
                    println!("Starting in Summarizer (Persona Layer) role");

                    // Create model registry client
                    // Create model registry API
                    let model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create persona layer manager
                    let persona_manager = PersonaManager::new();

                    // Create app with telemetry
                    let app = axum::Router::new().with_state(telemetry.clone());

                    // Start server
                    let addr = SocketAddr::new(config.server.host, config.server.port + 3);
                    println!("Persona Layer listening on {}", addr);

                    // Create TCP listener
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("Failed to bind to address");

                    println!("Health check endpoints available at:");
                    println!("  - /health");
                    println!("  - /readiness");
                    println!("  - /diagnostics");

                    // Start server
                    axum::serve(listener, app).await.expect("Server error");
                }
                Role::Audit => {
                    println!("Starting in Audit Controller role");

                    // Create app with telemetry
                    let app = axum::Router::new().with_state(telemetry.clone());

                    // Start server
                    let addr = SocketAddr::new(config.server.host, config.server.port + 4);
                    println!("Audit Controller listening on {}", addr);

                    // Create TCP listener
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("Failed to bind to address");

                    println!("Health check endpoints available at:");
                    println!("  - /health");
                    println!("  - /readiness");
                    println!("  - /diagnostics");

                    // Start server
                    axum::serve(listener, app).await.expect("Server error");
                }
                Role::All => {
                    println!("Starting all roles");

                    // Create model registry API
                    let model_registry_api = ModelRegistryApi::new();

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create a simple router config
                    let router_config =
                        intellirouter::modules::router_core::config::RouterConfig::default();

                    // Create model registry
                    let model_registry = Arc::new(ModelRegistry::new());

                    // Create router
                    let router = RouterImpl::new(router_config, model_registry)
                        .expect("Failed to create router");

                    // Create chain engine
                    let chain_engine = ChainEngine::new();

                    // Create RAG manager
                    let rag_manager = RagManager::new();

                    // Create persona layer manager
                    let persona_manager = PersonaManager::new();

                    // Create apps with telemetry
                    let router_app = axum::Router::new().with_state(telemetry.clone());

                    let chain_engine_app = axum::Router::new().with_state(telemetry.clone());

                    let rag_manager_app = axum::Router::new().with_state(telemetry.clone());

                    let persona_layer_app = axum::Router::new().with_state(telemetry.clone());

                    // Start servers
                    // Clone config for each async block to avoid move issues
                    let config1 = config.clone();
                    tokio::spawn(async move {
                        let addr = config1.server.socket_addr();
                        println!("Router listening on {}", addr);

                        // Create TCP listener
                        let listener = tokio::net::TcpListener::bind(&addr)
                            .await
                            .expect("Failed to bind to address");

                        println!("Health check endpoints available at:");
                        println!("  - /health");
                        println!("  - /readiness");
                        println!("  - /diagnostics");

                        // Start server
                        axum::serve(listener, router_app)
                            .await
                            .expect("Server error");
                    });

                    let config2 = config.clone();
                    tokio::spawn(async move {
                        let addr = SocketAddr::new(config2.server.host, config2.server.port + 1);
                        println!("Chain Engine listening on {}", addr);

                        // Create TCP listener
                        let listener = tokio::net::TcpListener::bind(&addr)
                            .await
                            .expect("Failed to bind to address");

                        println!("Health check endpoints available at:");
                        println!("  - /health");
                        println!("  - /readiness");
                        println!("  - /diagnostics");

                        // Start server
                        axum::serve(listener, chain_engine_app)
                            .await
                            .expect("Server error");
                    });

                    let config3 = config.clone();
                    tokio::spawn(async move {
                        let addr = SocketAddr::new(config3.server.host, config3.server.port + 2);
                        println!("RAG Manager listening on {}", addr);

                        // Create TCP listener
                        let listener = tokio::net::TcpListener::bind(&addr)
                            .await
                            .expect("Failed to bind to address");

                        println!("Health check endpoints available at:");
                        println!("  - /health");
                        println!("  - /readiness");
                        println!("  - /diagnostics");

                        // Start server
                        axum::serve(listener, rag_manager_app)
                            .await
                            .expect("Server error");
                    });

                    let config4 = config.clone();
                    tokio::spawn(async move {
                        let addr = SocketAddr::new(config4.server.host, config4.server.port + 3);
                        println!("Persona Layer listening on {}", addr);

                        // Create TCP listener
                        let listener = tokio::net::TcpListener::bind(&addr)
                            .await
                            .expect("Failed to bind to address");

                        println!("Health check endpoints available at:");
                        println!("  - /health");
                        println!("  - /readiness");
                        println!("  - /diagnostics");

                        // Start server
                        axum::serve(listener, persona_layer_app)
                            .await
                            .expect("Server error");
                    });

                    // Wait for Ctrl+C
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Failed to listen for Ctrl+C");
                    println!("Shutting down...");
                }
            }
        }
        Commands::GenerateConfig { output, env } => {
            println!("Generating configuration file for environment: {}", env);
            let config = Config::default();
            config
                .save_to_file(output.to_str().unwrap())
                .expect("Failed to write configuration file");
            println!("Configuration file generated at {:?}", output);
        }
    }
}
