use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use intellirouter::config::Config;
// Import public interfaces only
use intellirouter::modules::chain_engine::ChainEngine;
use intellirouter::modules::health::{
    create_chain_engine_health_manager, create_persona_layer_health_manager,
    create_rag_manager_health_manager, create_router_health_manager,
};
use intellirouter::modules::memory::{InMemoryBackend, MemoryManager};
use intellirouter::modules::model_registry::api::ModelRegistryApi;
use intellirouter::modules::model_registry::storage::ModelRegistry;
use intellirouter::modules::persona_layer::manager::PersonaManager;
use intellirouter::modules::rag_manager::manager::RagManager;
use intellirouter::modules::router_core::router::RouterImpl;
use intellirouter::modules::telemetry::telemetry::TelemetryManager;
use tracing::{error, info};

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

    // Create shutdown coordinator for graceful shutdown
    let shutdown_coordinator =
        Arc::new(intellirouter::modules::common::ShutdownCoordinator::new(1));

    // Set up signal handlers for graceful shutdown
    let shutdown_tx = shutdown_coordinator.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, initiating graceful shutdown...");
                if let Err(e) = shutdown_tx
                    .send_shutdown(intellirouter::modules::common::ShutdownSignal::Graceful)
                {
                    error!("Failed to send shutdown signal: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to listen for Ctrl+C: {}", e);
            }
        }
    });

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
                    let _model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create router
                    // Create a simple router config
                    let router_config =
                        intellirouter::modules::router_core::config::RouterConfig::default();

                    // Create model registry
                    let model_registry = Arc::new(ModelRegistry::new());

                    // Create router
                    let _router = RouterImpl::new(router_config.clone(), model_registry.clone())
                        .expect("Failed to create router");

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let _memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create chain engine
                    let _chain_engine = ChainEngine::new();

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

                    // Create health check manager
                    let redis_url = config.memory.redis_url.clone();
                    let health_manager = create_router_health_manager(
                        model_registry.clone(),
                        router_config.clone(),
                        redis_url,
                    );
                    let health_router = health_manager.create_router();

                    // Create router with routes
                    let app = intellirouter::modules::llm_proxy::server::create_router(app_state)
                        .merge(health_router);

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

                    // Create graceful shutdown future
                    let mut shutdown_rx = shutdown_coordinator.subscribe();
                    let completion_tx = shutdown_coordinator.completion_sender();

                    // Start server with graceful shutdown
                    let server = axum::serve(listener, app);
                    let graceful = server.with_graceful_shutdown(async move {
                        if let Ok(signal) = shutdown_rx.recv().await {
                            info!("Router received shutdown signal: {:?}", signal);
                        }
                        info!("Router shutting down gracefully...");
                    });

                    // Run the server and handle errors
                    if let Err(e) = graceful.await {
                        error!("Router server error: {}", e);
                    }

                    // Notify shutdown coordinator that we're done
                    if let Err(e) = completion_tx.send(()).await {
                        error!("Failed to send completion signal: {}", e);
                    }

                    info!("Router shutdown complete");
                }
                Role::Orchestrator => {
                    println!("Starting in Orchestrator (Chain Engine) role");

                    // Create model registry client
                    // Create model registry API
                    let _model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let _memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create rag manager
                    let _rag_manager = RagManager::new();

                    // Create persona layer manager
                    let _persona_manager = PersonaManager::new();

                    // Create chain engine
                    let chain_engine = Arc::new(ChainEngine::new());

                    // Create health check manager
                    let redis_url = config.memory.redis_url.clone();
                    let router_endpoint = Some(format!(
                        "http://{}:{}",
                        config.server.host, config.server.port
                    ));
                    let health_manager = create_chain_engine_health_manager(
                        chain_engine.clone(),
                        redis_url,
                        router_endpoint,
                    );
                    let health_router = health_manager.create_router();

                    // Create app with telemetry and health routes
                    let app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(health_router);

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

                    // Create graceful shutdown future
                    let mut shutdown_rx = shutdown_coordinator.subscribe();
                    let completion_tx = shutdown_coordinator.completion_sender();

                    // Start server with graceful shutdown
                    let server = axum::serve(listener, app);
                    let graceful = server.with_graceful_shutdown(async move {
                        if let Ok(signal) = shutdown_rx.recv().await {
                            info!("Chain Engine received shutdown signal: {:?}", signal);
                        }
                        info!("Chain Engine shutting down gracefully...");
                    });

                    // Run the server and handle errors
                    if let Err(e) = graceful.await {
                        error!("Chain Engine server error: {}", e);
                    }

                    // Notify shutdown coordinator that we're done
                    if let Err(e) = completion_tx.send(()).await {
                        error!("Failed to send completion signal: {}", e);
                    }

                    info!("Chain Engine shutdown complete");
                }
                Role::RagInjector => {
                    println!("Starting in RAG Injector (RAG Manager) role");

                    // Create model registry client
                    // Create model registry API
                    let _model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let _memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create RAG manager
                    let rag_manager = Arc::new(RagManager::new());

                    // Create health check manager
                    let redis_url = config.memory.redis_url.clone();
                    let router_endpoint = Some(format!(
                        "http://{}:{}",
                        config.server.host, config.server.port
                    ));
                    let vector_db_url = config.rag.vector_db_url.clone();
                    let health_manager = create_rag_manager_health_manager(
                        rag_manager.clone(),
                        redis_url,
                        router_endpoint,
                        vector_db_url,
                    );
                    let health_router = health_manager.create_router();

                    // Create app with telemetry and health routes
                    let app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(health_router);

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

                    // Create graceful shutdown future
                    let mut shutdown_rx = shutdown_coordinator.subscribe();
                    let completion_tx = shutdown_coordinator.completion_sender();

                    // Start server with graceful shutdown
                    let server = axum::serve(listener, app);
                    let graceful = server.with_graceful_shutdown(async move {
                        if let Ok(signal) = shutdown_rx.recv().await {
                            info!("RAG Manager received shutdown signal: {:?}", signal);
                        }
                        info!("RAG Manager shutting down gracefully...");
                    });

                    // Run the server and handle errors
                    if let Err(e) = graceful.await {
                        error!("RAG Manager server error: {}", e);
                    }

                    // Notify shutdown coordinator that we're done
                    if let Err(e) = completion_tx.send(()).await {
                        error!("Failed to send completion signal: {}", e);
                    }

                    info!("RAG Manager shutdown complete");
                }
                Role::Summarizer => {
                    println!("Starting in Summarizer (Persona Layer) role");

                    // Create model registry client
                    // Create model registry API
                    let _model_registry_api = Arc::new(ModelRegistryApi::new());

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let _memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create persona layer manager
                    let persona_manager = Arc::new(PersonaManager::new());

                    // Create health check manager
                    let redis_url = config.memory.redis_url.clone();
                    let router_endpoint = Some(format!(
                        "http://{}:{}",
                        config.server.host, config.server.port
                    ));
                    let health_manager = create_persona_layer_health_manager(
                        persona_manager.clone(),
                        redis_url,
                        router_endpoint,
                    );
                    let health_router = health_manager.create_router();

                    // Create app with telemetry and health routes
                    let app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(health_router);

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

                    // Create graceful shutdown future
                    let mut shutdown_rx = shutdown_coordinator.subscribe();
                    let completion_tx = shutdown_coordinator.completion_sender();

                    // Start server with graceful shutdown
                    let server = axum::serve(listener, app);
                    let graceful = server.with_graceful_shutdown(async move {
                        if let Ok(signal) = shutdown_rx.recv().await {
                            info!("Persona Layer received shutdown signal: {:?}", signal);
                        }
                        info!("Persona Layer shutting down gracefully...");
                    });

                    // Run the server and handle errors
                    if let Err(e) = graceful.await {
                        error!("Persona Layer server error: {}", e);
                    }

                    // Notify shutdown coordinator that we're done
                    if let Err(e) = completion_tx.send(()).await {
                        error!("Failed to send completion signal: {}", e);
                    }

                    info!("Persona Layer shutdown complete");
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

                    // Create graceful shutdown future
                    let mut shutdown_rx = shutdown_coordinator.subscribe();
                    let completion_tx = shutdown_coordinator.completion_sender();

                    // Start server with graceful shutdown
                    let server = axum::serve(listener, app);
                    let graceful = server.with_graceful_shutdown(async move {
                        if let Ok(signal) = shutdown_rx.recv().await {
                            info!("Audit Controller received shutdown signal: {:?}", signal);
                        }
                        info!("Audit Controller shutting down gracefully...");
                    });

                    // Run the server and handle errors
                    if let Err(e) = graceful.await {
                        error!("Audit Controller server error: {}", e);
                    }

                    // Notify shutdown coordinator that we're done
                    if let Err(e) = completion_tx.send(()).await {
                        error!("Failed to send completion signal: {}", e);
                    }

                    info!("Audit Controller shutdown complete");
                }
                Role::All => {
                    println!("Starting all roles");

                    // Update the shutdown coordinator to wait for all services
                    let shutdown_coordinator =
                        Arc::new(intellirouter::modules::common::ShutdownCoordinator::new(4));

                    // Create model registry API
                    let _model_registry_api = ModelRegistryApi::new();

                    // Create memory backend
                    let memory_backend = Arc::new(InMemoryBackend::new());

                    // Create memory manager with default window size
                    let _memory_manager = MemoryManager::new(memory_backend, 100);

                    // Create a simple router config
                    let router_config =
                        intellirouter::modules::router_core::config::RouterConfig::default();

                    // Create model registry
                    let model_registry = Arc::new(ModelRegistry::new());

                    // Create router
                    let _router = RouterImpl::new(router_config.clone(), model_registry.clone())
                        .expect("Failed to create router");

                    // Create chain engine
                    let chain_engine = Arc::new(ChainEngine::new());

                    // Create RAG manager
                    let rag_manager = Arc::new(RagManager::new());

                    // Create persona layer manager
                    let persona_manager = Arc::new(PersonaManager::new());

                    // Create resilient clients for inter-service communication
                    // These will be used when services need to communicate with each other
                    let _resilient_clients =
                        match intellirouter::modules::ipc::utils::create_all_resilient_clients(
                            &config,
                        )
                        .await
                        {
                            Ok(clients) => {
                                info!("Successfully created resilient clients for inter-service communication");
                                Some(clients)
                            }
                            Err(e) => {
                                error!("Failed to create resilient clients: {}", e);
                                None
                            }
                        };

                    // Configure retry policy for inter-service communication
                    let _retry_policy = intellirouter::modules::router_core::retry::RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 100,
                        backoff_factor: 2.0,
                        max_retries: 3,
                        max_interval_ms: 5000,
                    };

                    // Configure circuit breaker for inter-service communication
                    let _circuit_breaker_config =
                        intellirouter::modules::router_core::retry::CircuitBreakerConfig {
                            failure_threshold: 5,
                            success_threshold: 3,
                            reset_timeout_ms: 30000, // 30 seconds
                            enabled: true,
                        };

                    // Configure retryable error categories for inter-service communication
                    let mut _retryable_errors = std::collections::HashSet::new();
                    _retryable_errors
                        .insert(intellirouter::modules::router_core::retry::ErrorCategory::Network);
                    _retryable_errors
                        .insert(intellirouter::modules::router_core::retry::ErrorCategory::Timeout);
                    _retryable_errors.insert(
                        intellirouter::modules::router_core::retry::ErrorCategory::RateLimit,
                    );
                    _retryable_errors
                        .insert(intellirouter::modules::router_core::retry::ErrorCategory::Server);

                    // Create health check managers
                    let redis_url = config.memory.redis_url.clone();
                    let vector_db_url = config.rag.vector_db_url.clone();

                    // Router health check
                    let router_health_manager = create_router_health_manager(
                        model_registry.clone(),
                        router_config.clone(),
                        redis_url.clone(),
                    );
                    let router_health_router = router_health_manager.create_router();

                    // Chain Engine health check
                    let router_endpoint = Some(format!(
                        "http://{}:{}",
                        config.server.host, config.server.port
                    ));
                    let chain_engine_health_manager = create_chain_engine_health_manager(
                        chain_engine.clone(),
                        redis_url.clone(),
                        router_endpoint.clone(),
                    );
                    let chain_engine_health_router = chain_engine_health_manager.create_router();

                    // RAG Manager health check
                    let rag_manager_health_manager = create_rag_manager_health_manager(
                        rag_manager.clone(),
                        redis_url.clone(),
                        router_endpoint.clone(),
                        vector_db_url,
                    );
                    let rag_manager_health_router = rag_manager_health_manager.create_router();

                    // Persona Layer health check
                    let persona_layer_health_manager = create_persona_layer_health_manager(
                        persona_manager.clone(),
                        redis_url.clone(),
                        router_endpoint.clone(),
                    );
                    let persona_layer_health_router = persona_layer_health_manager.create_router();

                    // Create apps with telemetry and health routes
                    let router_app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(router_health_router);

                    let chain_engine_app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(chain_engine_health_router);

                    let rag_manager_app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(rag_manager_health_router);

                    let persona_layer_app = axum::Router::new()
                        .with_state(telemetry.clone())
                        .merge(persona_layer_health_router);

                    // Start servers
                    // Clone config and shutdown_coordinator for each async block to avoid move issues
                    let config1 = config.clone();
                    let shutdown_coordinator1 = shutdown_coordinator.clone();
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

                        // Create graceful shutdown future
                        let mut shutdown_rx = shutdown_coordinator1.subscribe();
                        let completion_tx = shutdown_coordinator1.completion_sender();

                        // Start server with graceful shutdown
                        let server = axum::serve(listener, router_app);
                        let graceful = server.with_graceful_shutdown(async move {
                            if let Ok(signal) = shutdown_rx.recv().await {
                                info!("Router received shutdown signal: {:?}", signal);
                            }
                            info!("Router shutting down gracefully...");
                        });

                        // Run the server and handle errors
                        if let Err(e) = graceful.await {
                            error!("Router server error: {}", e);
                        }

                        // Notify shutdown coordinator that we're done
                        if let Err(e) = completion_tx.send(()).await {
                            error!("Failed to send completion signal: {}", e);
                        }

                        info!("Router shutdown complete");
                    });

                    let config2 = config.clone();
                    let shutdown_coordinator2 = shutdown_coordinator.clone();
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

                        // Create graceful shutdown future
                        let mut shutdown_rx = shutdown_coordinator2.subscribe();
                        let completion_tx = shutdown_coordinator2.completion_sender();

                        // Start server with graceful shutdown
                        let server = axum::serve(listener, chain_engine_app);
                        let graceful = server.with_graceful_shutdown(async move {
                            if let Ok(signal) = shutdown_rx.recv().await {
                                info!("Chain Engine received shutdown signal: {:?}", signal);
                            }
                            info!("Chain Engine shutting down gracefully...");
                        });

                        // Run the server and handle errors
                        if let Err(e) = graceful.await {
                            error!("Chain Engine server error: {}", e);
                        }

                        // Notify shutdown coordinator that we're done
                        if let Err(e) = completion_tx.send(()).await {
                            error!("Failed to send completion signal: {}", e);
                        }

                        info!("Chain Engine shutdown complete");
                    });

                    let config3 = config.clone();
                    let shutdown_coordinator3 = shutdown_coordinator.clone();
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

                        // Create graceful shutdown future
                        let mut shutdown_rx = shutdown_coordinator3.subscribe();
                        let completion_tx = shutdown_coordinator3.completion_sender();

                        // Start server with graceful shutdown
                        let server = axum::serve(listener, rag_manager_app);
                        let graceful = server.with_graceful_shutdown(async move {
                            if let Ok(signal) = shutdown_rx.recv().await {
                                info!("RAG Manager received shutdown signal: {:?}", signal);
                            }
                            info!("RAG Manager shutting down gracefully...");
                        });

                        // Run the server and handle errors
                        if let Err(e) = graceful.await {
                            error!("RAG Manager server error: {}", e);
                        }

                        // Notify shutdown coordinator that we're done
                        if let Err(e) = completion_tx.send(()).await {
                            error!("Failed to send completion signal: {}", e);
                        }

                        info!("RAG Manager shutdown complete");
                    });

                    let config4 = config.clone();
                    let shutdown_coordinator4 = shutdown_coordinator.clone();
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

                        // Create graceful shutdown future
                        let mut shutdown_rx = shutdown_coordinator4.subscribe();
                        let completion_tx = shutdown_coordinator4.completion_sender();

                        // Start server with graceful shutdown
                        let server = axum::serve(listener, persona_layer_app);
                        let graceful = server.with_graceful_shutdown(async move {
                            if let Ok(signal) = shutdown_rx.recv().await {
                                info!("Persona Layer received shutdown signal: {:?}", signal);
                            }
                            info!("Persona Layer shutting down gracefully...");
                        });

                        // Run the server and handle errors
                        if let Err(e) = graceful.await {
                            error!("Persona Layer server error: {}", e);
                        }

                        // Notify shutdown coordinator that we're done
                        if let Err(e) = completion_tx.send(()).await {
                            error!("Failed to send completion signal: {}", e);
                        }

                        info!("Persona Layer shutdown complete");
                    });

                    // Set up signal handlers for graceful shutdown
                    let shutdown_tx = shutdown_coordinator.clone();
                    let _shutdown_coordinator5 = shutdown_coordinator.clone();
                    tokio::spawn(async move {
                        match tokio::signal::ctrl_c().await {
                            Ok(()) => {
                                info!("Received Ctrl+C, initiating graceful shutdown...");
                                if let Err(e) = shutdown_tx.send_shutdown(
                                    intellirouter::modules::common::ShutdownSignal::Graceful,
                                ) {
                                    error!("Failed to send shutdown signal: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to listen for Ctrl+C: {}", e);
                            }
                        }
                    });

                    // Wait for all services to shut down using the shared method
                    match intellirouter::modules::common::ShutdownCoordinator::wait_for_completion_shared(
                        &shutdown_coordinator,
                        30000
                    ).await {
                        Ok(()) => {
                            info!("All services shut down successfully");
                        }
                        Err(_) => {
                            error!("Timed out waiting for services to shut down");
                        }
                    }

                    println!("Shutdown complete");
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
