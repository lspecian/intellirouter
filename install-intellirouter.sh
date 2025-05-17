#!/bin/bash
# IntelliRouter Physical Deployment Installation Script
# This script automates the installation of IntelliRouter components on physical nodes

set -e

# Default values
INSTALL_DIR="/opt/intellirouter"
ROLE=""
CONFIG_DIR="./config"
BUILD_FROM_SOURCE=false
CREATE_USER=true
INSTALL_DEPENDENCIES=true
SETUP_SYSTEMD=true

# Display help message
function show_help {
    echo "IntelliRouter Installation Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -r, --role ROLE            Role to install (router, orchestrator, rag-manager, persona-layer, all)"
    echo "  -d, --install-dir DIR      Installation directory (default: /opt/intellirouter)"
    echo "  -c, --config-dir DIR       Configuration directory (default: ./config)"
    echo "  -b, --build-from-source    Build IntelliRouter from source (default: false)"
    echo "  --no-create-user           Don't create intellirouter user"
    echo "  --no-dependencies          Don't install system dependencies"
    echo "  --no-systemd               Don't setup systemd services"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 --role router --install-dir /opt/intellirouter"
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -r|--role)
            ROLE="$2"
            shift
            shift
            ;;
        -d|--install-dir)
            INSTALL_DIR="$2"
            shift
            shift
            ;;
        -c|--config-dir)
            CONFIG_DIR="$2"
            shift
            shift
            ;;
        -b|--build-from-source)
            BUILD_FROM_SOURCE=true
            shift
            ;;
        --no-create-user)
            CREATE_USER=false
            shift
            ;;
        --no-dependencies)
            INSTALL_DEPENDENCIES=false
            shift
            ;;
        --no-systemd)
            SETUP_SYSTEMD=false
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Validate role
if [[ -z "$ROLE" ]]; then
    echo "Error: Role must be specified with -r or --role"
    show_help
fi

if [[ "$ROLE" != "router" && "$ROLE" != "orchestrator" && "$ROLE" != "rag-manager" && "$ROLE" != "persona-layer" && "$ROLE" != "all" ]]; then
    echo "Error: Invalid role. Must be one of: router, orchestrator, rag-manager, persona-layer, all"
    exit 1
fi

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
    exit 1
fi

echo "=== IntelliRouter Installation ==="
echo "Role: $ROLE"
echo "Installation directory: $INSTALL_DIR"
echo "Configuration directory: $CONFIG_DIR"
echo "Build from source: $BUILD_FROM_SOURCE"
echo "Create user: $CREATE_USER"
echo "Install dependencies: $INSTALL_DEPENDENCIES"
echo "Setup systemd: $SETUP_SYSTEMD"
echo ""

# Confirm installation
read -p "Continue with installation? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation cancelled"
    exit 1
fi

# Install system dependencies
if [[ "$INSTALL_DEPENDENCIES" = true ]]; then
    echo "=== Installing system dependencies ==="
    apt update
    apt install -y build-essential pkg-config libssl-dev curl git
    
    # Install Rust if building from source
    if [[ "$BUILD_FROM_SOURCE" = true ]]; then
        if ! command -v rustc &> /dev/null; then
            echo "Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source $HOME/.cargo/env
        fi
    fi
    
    # Install Redis if needed
    if [[ "$ROLE" = "all" ]]; then
        echo "Installing Redis..."
        apt install -y redis-server
        systemctl enable redis-server
        systemctl start redis-server
    fi
    
    # Install Docker if needed for ChromaDB
    if [[ "$ROLE" = "rag-manager" || "$ROLE" = "all" ]]; then
        echo "Installing Docker for ChromaDB..."
        apt install -y docker.io
        systemctl enable --now docker
    fi
fi

# Create intellirouter user
if [[ "$CREATE_USER" = true ]]; then
    echo "=== Creating intellirouter user ==="
    if ! id -u intellirouter &>/dev/null; then
        useradd -m -s /bin/bash intellirouter
    fi
fi

# Create directory structure
echo "=== Creating directory structure ==="
mkdir -p $INSTALL_DIR/{bin,config,data,logs,env}
mkdir -p $INSTALL_DIR/data/{documents,personas}
mkdir -p $INSTALL_DIR/logs/{router,orchestrator,rag-manager,persona-layer}

# Set permissions
chown -R intellirouter:intellirouter $INSTALL_DIR

# Install IntelliRouter
echo "=== Installing IntelliRouter ==="
if [[ "$BUILD_FROM_SOURCE" = true ]]; then
    echo "Building from source..."
    # Clone repository if it doesn't exist
    if [[ ! -d "intellirouter" ]]; then
        git clone https://github.com/intellirouter/intellirouter.git
    fi
    
    cd intellirouter
    source $HOME/.cargo/env
    cargo build --release
    
    # Copy binary
    cp target/release/intellirouter $INSTALL_DIR/bin/
else
    echo "Downloading pre-built binary..."
    # This is a placeholder - replace with actual download URL
    curl -L -o $INSTALL_DIR/bin/intellirouter https://github.com/intellirouter/intellirouter/releases/latest/download/intellirouter
    chmod +x $INSTALL_DIR/bin/intellirouter
fi

# Copy configuration files
echo "=== Copying configuration files ==="
cp -r $CONFIG_DIR/* $INSTALL_DIR/config/

# Create environment files
echo "=== Creating environment files ==="

# Router environment
if [[ "$ROLE" = "router" || "$ROLE" = "all" ]]; then
    cat > $INSTALL_DIR/env/router.env << EOF
INTELLIROUTER_ENVIRONMENT=production
INTELLIROUTER__SERVER__HOST=0.0.0.0
INTELLIROUTER__SERVER__PORT=8080
INTELLIROUTER__TELEMETRY__LOG_LEVEL=info
INTELLIROUTER__MEMORY__BACKEND_TYPE=redis
INTELLIROUTER__MEMORY__REDIS_URL=redis://localhost:6379
INTELLIROUTER__IPC__SECURITY__ENABLED=true
INTELLIROUTER__IPC__SECURITY__TOKEN=change_this_to_a_secure_token
# API Keys (set as needed)
# ANTHROPIC_API_KEY=your_anthropic_key
# OPENAI_API_KEY=your_openai_key
# GOOGLE_API_KEY=your_google_key
# MISTRAL_API_KEY=your_mistral_key
EOF
fi

# Orchestrator environment
if [[ "$ROLE" = "orchestrator" || "$ROLE" = "all" ]]; then
    cat > $INSTALL_DIR/env/orchestrator.env << EOF
INTELLIROUTER_ENVIRONMENT=production
INTELLIROUTER__SERVER__HOST=0.0.0.0
INTELLIROUTER__SERVER__PORT=8080
INTELLIROUTER__TELEMETRY__LOG_LEVEL=info
INTELLIROUTER__MEMORY__BACKEND_TYPE=redis
INTELLIROUTER__MEMORY__REDIS_URL=redis://localhost:6379
INTELLIROUTER__IPC__SECURITY__ENABLED=true
INTELLIROUTER__IPC__SECURITY__TOKEN=change_this_to_a_secure_token
# Service discovery
INTELLIROUTER__IPC__ROUTER_ENDPOINT=http://localhost:8080
EOF
fi

# RAG Manager environment
if [[ "$ROLE" = "rag-manager" || "$ROLE" = "all" ]]; then
    cat > $INSTALL_DIR/env/rag-manager.env << EOF
INTELLIROUTER_ENVIRONMENT=production
INTELLIROUTER__SERVER__HOST=0.0.0.0
INTELLIROUTER__SERVER__PORT=8080
INTELLIROUTER__TELEMETRY__LOG_LEVEL=info
INTELLIROUTER__MEMORY__BACKEND_TYPE=redis
INTELLIROUTER__MEMORY__REDIS_URL=redis://localhost:6379
INTELLIROUTER__IPC__SECURITY__ENABLED=true
INTELLIROUTER__IPC__SECURITY__TOKEN=change_this_to_a_secure_token
INTELLIROUTER__RAG__ENABLED=true
INTELLIROUTER__RAG__VECTOR_DB_URL=http://localhost:8000
# Service discovery
INTELLIROUTER__IPC__ROUTER_ENDPOINT=http://localhost:8080
EOF
fi

# Persona Layer environment
if [[ "$ROLE" = "persona-layer" || "$ROLE" = "all" ]]; then
    cat > $INSTALL_DIR/env/persona-layer.env << EOF
INTELLIROUTER_ENVIRONMENT=production
INTELLIROUTER__SERVER__HOST=0.0.0.0
INTELLIROUTER__SERVER__PORT=8080
INTELLIROUTER__TELEMETRY__LOG_LEVEL=info
INTELLIROUTER__MEMORY__BACKEND_TYPE=redis
INTELLIROUTER__MEMORY__REDIS_URL=redis://localhost:6379
INTELLIROUTER__IPC__SECURITY__ENABLED=true
INTELLIROUTER__IPC__SECURITY__TOKEN=change_this_to_a_secure_token
INTELLIROUTER__PERSONA_LAYER__ENABLED=true
# Service discovery
INTELLIROUTER__IPC__ROUTER_ENDPOINT=http://localhost:8080
EOF
fi

# Set permissions for environment files
chmod 600 $INSTALL_DIR/env/*.env
chown intellirouter:intellirouter $INSTALL_DIR/env/*.env

# Setup systemd services
if [[ "$SETUP_SYSTEMD" = true ]]; then
    echo "=== Setting up systemd services ==="
    
    # Router service
    if [[ "$ROLE" = "router" || "$ROLE" = "all" ]]; then
        cat > /etc/systemd/system/intellirouter-router.service << EOF
[Unit]
Description=IntelliRouter Router Service
After=network.target
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/env/router.env
ExecStart=$INSTALL_DIR/bin/intellirouter run --role Router --config $INSTALL_DIR/config/router.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:$INSTALL_DIR/logs/router/stdout.log
StandardError=append:$INSTALL_DIR/logs/router/stderr.log

# Security settings
ProtectSystem=full
PrivateTmp=true
NoNewPrivileges=true
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
        systemctl enable intellirouter-router
    fi
    
    # Orchestrator service
    if [[ "$ROLE" = "orchestrator" || "$ROLE" = "all" ]]; then
        cat > /etc/systemd/system/intellirouter-orchestrator.service << EOF
[Unit]
Description=IntelliRouter Orchestrator Service (Chain Engine)
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/env/orchestrator.env
ExecStart=$INSTALL_DIR/bin/intellirouter run --role ChainEngine --config $INSTALL_DIR/config/orchestrator.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:$INSTALL_DIR/logs/orchestrator/stdout.log
StandardError=append:$INSTALL_DIR/logs/orchestrator/stderr.log

# Security settings
ProtectSystem=full
PrivateTmp=true
NoNewPrivileges=true
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
        systemctl enable intellirouter-orchestrator
    fi
    
    # RAG Manager service
    if [[ "$ROLE" = "rag-manager" || "$ROLE" = "all" ]]; then
        cat > /etc/systemd/system/intellirouter-rag-manager.service << EOF
[Unit]
Description=IntelliRouter RAG Manager Service
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/env/rag-manager.env
ExecStart=$INSTALL_DIR/bin/intellirouter run --role RagManager --config $INSTALL_DIR/config/rag-manager.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:$INSTALL_DIR/logs/rag-manager/stdout.log
StandardError=append:$INSTALL_DIR/logs/rag-manager/stderr.log

# Security settings
ProtectSystem=full
PrivateTmp=true
NoNewPrivileges=true
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
        systemctl enable intellirouter-rag-manager
        
        # Start ChromaDB if needed
        echo "Starting ChromaDB container..."
        docker run -d \
          --name chromadb \
          --restart unless-stopped \
          -p 8000:8000 \
          -v $INSTALL_DIR/data/chromadb:/chroma/chroma \
          ghcr.io/chroma-core/chroma:latest
    fi
    
    # Persona Layer service
    if [[ "$ROLE" = "persona-layer" || "$ROLE" = "all" ]]; then
        cat > /etc/systemd/system/intellirouter-persona-layer.service << EOF
[Unit]
Description=IntelliRouter Persona Layer Service (Summarizer)
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/env/persona-layer.env
ExecStart=$INSTALL_DIR/bin/intellirouter run --role PersonaLayer --config $INSTALL_DIR/config/persona-layer.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:$INSTALL_DIR/logs/persona-layer/stdout.log
StandardError=append:$INSTALL_DIR/logs/persona-layer/stderr.log

# Security settings
ProtectSystem=full
PrivateTmp=true
NoNewPrivileges=true
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
        systemctl enable intellirouter-persona-layer
    fi
    
    # Setup log rotation
    cat > /etc/logrotate.d/intellirouter << EOF
$INSTALL_DIR/logs/*/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 intellirouter intellirouter
    sharedscripts
    postrotate
        systemctl reload intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer >/dev/null 2>&1 || true
    endscript
}
EOF
fi

echo "=== Installation Complete ==="
echo ""
echo "IntelliRouter has been installed to $INSTALL_DIR"
echo ""
echo "To start the services:"

if [[ "$ROLE" = "router" || "$ROLE" = "all" ]]; then
    echo "  sudo systemctl start intellirouter-router"
fi

if [[ "$ROLE" = "orchestrator" || "$ROLE" = "all" ]]; then
    echo "  sudo systemctl start intellirouter-orchestrator"
fi

if [[ "$ROLE" = "rag-manager" || "$ROLE" = "all" ]]; then
    echo "  sudo systemctl start intellirouter-rag-manager"
fi

if [[ "$ROLE" = "persona-layer" || "$ROLE" = "all" ]]; then
    echo "  sudo systemctl start intellirouter-persona-layer"
fi

echo ""
echo "IMPORTANT: Before starting the services, update the environment files in $INSTALL_DIR/env/"
echo "to set the correct API keys and service endpoints for your deployment."
echo ""
echo "To check service status:"
echo "  sudo systemctl status intellirouter-*"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u intellirouter-router -f"
echo ""