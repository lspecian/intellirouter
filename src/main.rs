use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use intellirouter::config::Config;
use intellirouter::modules::authz::routes as authz_routes;
use intellirouter::modules::chain_engine::engine::ChainEngine;
use intellirouter::modules::ipc::chain_engine::ChainEngineClient;
use intellirouter::modules::ipc::memory::client::MemoryClient;
use intellirouter::modules::ipc::memory::service::MemoryService;
use intellirouter::modules::ipc::model_registry::ModelRegistryClient;
use intellirouter::modules::ipc::persona_layer::PersonaLayerClient;
use intellirouter::modules::ipc::rag_manager::RagManagerClient;
use intellirouter::modules::llm_proxy::routes as llm_proxy_routes;
use intellirouter::modules::llm_proxy::server::LlmProxyServer;
use intellirouter::modules::memory::manager::MemoryManager;
use intellirouter::modules::model_registry::api::ModelRegistryApi;
use intellirouter::modules::persona_layer::manager::PersonaLayerManager;
use intellirouter::modules::rag_manager::manager::RagManager;
use intellirouter::modules::router_core::router::Router;
use intellirouter::modules::telemetry::middleware::telemetry_middleware;
use intellirouter::modules::telemetry::telemetry::{init_telemetry, TelemetryManager};
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
            let config = Config::from_file(config_path).expect("Failed to load configuration");

            // Initialize telemetry with configuration
            let metrics_addr: SocketAddr = config.metrics.socket_addr();
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
                    let model_registry_client =
                        ModelRegistryClient::new(&config.model_registry.endpoint)
                            .await
                            .expect("Failed to create model registry client");

                    // Create router
                    let router = Router::new(model_registry_client.clone());

                    // Create memory client
                    let memory_client = MemoryClient::new(&config.memory.endpoint)
                        .await
                        .expect("Failed to create memory client");

                    // Create chain engine client
                    let chain_engine_client = ChainEngineClient::new(&config.chain_engine.endpoint)
                        .await
                        .expect("Failed to create chain engine client");

                    // Create LLM proxy server
                    let llm_proxy_server =
                        LlmProxyServer::new(router, memory_client, chain_engine_client);

                    // Create app with telemetry
                    let app = axum::Router::new()
                        .merge(llm_proxy_routes::router(llm_proxy_server))
                        .merge(authz_routes::router())
                        .with_state(telemetry.clone());

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
                    let model_registry_client =
                        ModelRegistryClient::new(&config.model_registry.endpoint)
                            .await
                            .expect("Failed to create model registry client");

                    // Create memory client
                    let memory_client = MemoryClient::new(&config.memory.endpoint)
                        .await
                        .expect("Failed to create memory client");

                    // Create rag manager client
                    let rag_manager_client = RagManagerClient::new(&config.rag_manager.endpoint)
                        .await
                        .expect("Failed to create rag manager client");

                    // Create persona layer client
                    let persona_layer_client =
                        PersonaLayerClient::new(&config.persona_layer.endpoint)
                            .await
                            .expect("Failed to create persona layer client");

                    // Create chain engine
                    let chain_engine = ChainEngine::new(
                        model_registry_client,
                        memory_client,
                        rag_manager_client,
                        persona_layer_client,
                    );

                    // Create app with telemetry
                    let app = axum::Router::new()
                        .merge(intellirouter::modules::chain_engine::routes::router(
                            chain_engine,
                        ))
                        .with_state(telemetry.clone());

                    // Start server
                    let addr = config.chain_engine.socket_addr();
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
                    let model_registry_client =
                        ModelRegistryClient::new(&config.model_registry.endpoint)
                            .await
                            .expect("Failed to create model registry client");

                    // Create memory client
                    let memory_client = MemoryClient::new(&config.memory.endpoint)
                        .await
                        .expect("Failed to create memory client");

                    // Create RAG manager
                    let rag_manager = RagManager::new(model_registry_client, memory_client);

                    // Create app with telemetry
                    let app = axum::Router::new()
                        .merge(intellirouter::modules::rag_manager::routes::router(
                            rag_manager,
                        ))
                        .with_state(telemetry.clone());

                    // Start server
                    let addr = config.rag_manager.socket_addr();
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
                    let model_registry_client =
                        ModelRegistryClient::new(&config.model_registry.endpoint)
                            .await
                            .expect("Failed to create model registry client");

                    // Create memory client
                    let memory_client = MemoryClient::new(&config.memory.endpoint)
                        .await
                        .expect("Failed to create memory client");

                    // Create persona layer manager
                    let persona_layer_manager =
                        PersonaLayerManager::new(model_registry_client, memory_client);

                    // Create app with telemetry
                    let app = axum::Router::new()
                        .merge(intellirouter::modules::persona_layer::routes::router(
                            persona_layer_manager,
                        ))
                        .with_state(telemetry.clone());

                    // Start server
                    let addr = config.persona_layer.socket_addr();
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
                    let app = axum::Router::new()
                        .merge(intellirouter::modules::audit::routes::router())
                        .with_state(telemetry.clone());

                    // Start server
                    let addr = config.audit.socket_addr();
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

                    // Create memory manager
                    let memory_manager =
                        MemoryManager::new(&config.memory.backend_type, &config.memory.redis_url);

                    // Create memory service
                    let memory_service = MemoryService::new(memory_manager);

                    // Start memory service
                    let (tx, rx) = mpsc::channel(100);
                    tokio::spawn(async move {
                        memory_service.run(rx).await;
                    });

                    // Create memory client
                    let memory_client = MemoryClient::from_channel(tx);

                    // Create router
                    let router = Router::new(model_registry_api.clone());

                    // Create chain engine
                    let chain_engine = ChainEngine::new(
                        model_registry_api.clone(),
                        memory_client.clone(),
                        RagManagerClient::new(&config.rag_manager.endpoint)
                            .await
                            .expect("Failed to create rag manager client"),
                        PersonaLayerClient::new(&config.persona_layer.endpoint)
                            .await
                            .expect("Failed to create persona layer client"),
                    );

                    // Create LLM proxy server
                    let llm_proxy_server = LlmProxyServer::new(
                        router,
                        memory_client.clone(),
                        ChainEngineClient::new(&config.chain_engine.endpoint)
                            .await
                            .expect("Failed to create chain engine client"),
                    );

                    // Create RAG manager
                    let rag_manager =
                        RagManager::new(model_registry_api.clone(), memory_client.clone());

                    // Create persona layer manager
                    let persona_layer_manager =
                        PersonaLayerManager::new(model_registry_api, memory_client);

                    // Create apps with telemetry
                    let router_app = axum::Router::new()
                        .merge(llm_proxy_routes::router(llm_proxy_server))
                        .merge(authz_routes::router())
                        .with_state(telemetry.clone());

                    let chain_engine_app = axum::Router::new()
                        .merge(intellirouter::modules::chain_engine::routes::router(
                            chain_engine,
                        ))
                        .with_state(telemetry.clone());

                    let rag_manager_app = axum::Router::new()
                        .merge(intellirouter::modules::rag_manager::routes::router(
                            rag_manager,
                        ))
                        .with_state(telemetry.clone());

                    let persona_layer_app = axum::Router::new()
                        .merge(intellirouter::modules::persona_layer::routes::router(
                            persona_layer_manager,
                        ))
                        .with_state(telemetry.clone());

                    // Start servers
                    tokio::spawn(async move {
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
                        axum::serve(listener, router_app)
                            .await
                            .expect("Server error");
                    });

                    tokio::spawn(async move {
                        let addr = config.chain_engine.socket_addr();
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

                    tokio::spawn(async move {
                        let addr = config.rag_manager.socket_addr();
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

                    tokio::spawn(async move {
                        let addr = config.persona_layer.socket_addr();
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
                .to_file(&output)
                .expect("Failed to write configuration file");
            println!("Configuration file generated at {:?}", output);
        }
    }
}
