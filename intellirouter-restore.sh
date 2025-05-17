#!/bin/bash
# IntelliRouter Restore Script
# This script restores an IntelliRouter backup

set -e

# Default values
INSTALL_DIR="/opt/intellirouter"
BACKUP_FILE=""
RESTORE_CONFIG=true
RESTORE_DATA=true
RESTORE_REDIS=true
RESTORE_CHROMADB=true
RESTART_SERVICES=true
TEMP_DIR="/tmp/intellirouter-restore"

# Display help message
function show_help {
    echo "IntelliRouter Restore Script"
    echo ""
    echo "Usage: $0 [options] <backup_file>"
    echo ""
    echo "Options:"
    echo "  -d, --install-dir DIR      Installation directory (default: /opt/intellirouter)"
    echo "  --no-config                Don't restore configuration"
    echo "  --no-data                  Don't restore data"
    echo "  --no-redis                 Don't restore Redis data"
    echo "  --no-chromadb              Don't restore ChromaDB data"
    echo "  --no-restart               Don't restart services after restore"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 --install-dir /opt/intellirouter /opt/intellirouter/backups/2025-05-17.tar.gz"
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -d|--install-dir)
            INSTALL_DIR="$2"
            shift
            shift
            ;;
        --no-config)
            RESTORE_CONFIG=false
            shift
            ;;
        --no-data)
            RESTORE_DATA=false
            shift
            ;;
        --no-redis)
            RESTORE_REDIS=false
            shift
            ;;
        --no-chromadb)
            RESTORE_CHROMADB=false
            shift
            ;;
        --no-restart)
            RESTART_SERVICES=false
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            if [[ -z "$BACKUP_FILE" ]]; then
                BACKUP_FILE="$1"
                shift
            else
                echo "Unknown option: $1"
                show_help
            fi
            ;;
    esac
done

# Validate backup file
if [[ -z "$BACKUP_FILE" ]]; then
    echo "Error: Backup file must be specified"
    show_help
fi

if [[ ! -e "$BACKUP_FILE" ]]; then
    echo "Error: Backup file does not exist: $BACKUP_FILE"
    exit 1
fi

echo "=== IntelliRouter Restore ==="
echo "Installation directory: $INSTALL_DIR"
echo "Backup file: $BACKUP_FILE"
echo "Restore configuration: $RESTORE_CONFIG"
echo "Restore data: $RESTORE_DATA"
echo "Restore Redis: $RESTORE_REDIS"
echo "Restore ChromaDB: $RESTORE_CHROMADB"
echo "Restart services: $RESTART_SERVICES"
echo ""

# Confirm restore
echo "WARNING: This will overwrite existing data in $INSTALL_DIR"
read -p "Continue with restore? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Restore cancelled"
    exit 1
fi

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
    exit 1
fi

# Create temporary directory
mkdir -p $TEMP_DIR

# Extract backup
echo "Extracting backup..."
if [[ "$BACKUP_FILE" == *.tar.gz ]]; then
    tar -xzf $BACKUP_FILE -C $TEMP_DIR
    # Find the actual backup directory inside the temp dir
    BACKUP_DIR=$(find $TEMP_DIR -type d -maxdepth 1 | grep -v "^$TEMP_DIR$" | head -1)
    if [[ -z "$BACKUP_DIR" ]]; then
        # If no subdirectory found, use temp dir itself
        BACKUP_DIR=$TEMP_DIR
    fi
else
    # Backup file is a directory
    BACKUP_DIR=$BACKUP_FILE
fi

# Stop services
if [[ "$RESTART_SERVICES" = true ]]; then
    echo "Stopping IntelliRouter services..."
    systemctl stop intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer || true
fi

# Restore configuration
if [[ "$RESTORE_CONFIG" = true ]]; then
    echo "Restoring configuration..."
    if [[ -d "$BACKUP_DIR/config" ]]; then
        cp -r $BACKUP_DIR/config/* $INSTALL_DIR/config/
    else
        echo "Warning: Configuration directory not found in backup"
    fi
    
    if [[ -d "$BACKUP_DIR/env" ]]; then
        cp -r $BACKUP_DIR/env/* $INSTALL_DIR/env/
    else
        echo "Warning: Environment directory not found in backup"
    fi
fi

# Restore data
if [[ "$RESTORE_DATA" = true ]]; then
    echo "Restoring data..."
    if [[ -d "$BACKUP_DIR/data" ]]; then
        # Backup current data directory
        if [[ -d "$INSTALL_DIR/data" ]]; then
            mv $INSTALL_DIR/data $INSTALL_DIR/data.bak.$(date +%Y%m%d%H%M%S)
        fi
        
        # Create new data directory
        mkdir -p $INSTALL_DIR/data
        
        # Copy data excluding ChromaDB if needed
        if [[ "$RESTORE_CHROMADB" = false ]]; then
            find $BACKUP_DIR/data -mindepth 1 -maxdepth 1 -not -name "chromadb" -exec cp -r {} $INSTALL_DIR/data/ \;
        else
            cp -r $BACKUP_DIR/data/* $INSTALL_DIR/data/
        fi
    else
        echo "Warning: Data directory not found in backup"
    fi
fi

# Restore Redis data
if [[ "$RESTORE_REDIS" = true ]]; then
    echo "Restoring Redis data..."
    if [[ -f "$BACKUP_DIR/redis-dump.rdb" ]]; then
        systemctl stop redis-server || true
        cp $BACKUP_DIR/redis-dump.rdb /var/lib/redis/dump.rdb
        chown redis:redis /var/lib/redis/dump.rdb
        systemctl start redis-server || true
    else
        echo "Warning: Redis dump file not found in backup"
    fi
fi

# Restore ChromaDB data
if [[ "$RESTORE_CHROMADB" = true ]]; then
    echo "Restoring ChromaDB data..."
    if [[ -d "$BACKUP_DIR/chromadb" ]]; then
        # Stop ChromaDB container
        if docker ps -a | grep -q chromadb; then
            docker stop chromadb || true
        fi
        
        # Create ChromaDB data directory if it doesn't exist
        mkdir -p $INSTALL_DIR/data/chromadb
        
        # Copy ChromaDB data
        cp -r $BACKUP_DIR/chromadb/* $INSTALL_DIR/data/chromadb/
        
        # Start ChromaDB container
        if docker ps -a | grep -q chromadb; then
            docker start chromadb || true
        else
            echo "Warning: ChromaDB container not found, starting a new one..."
            docker run -d \
              --name chromadb \
              --restart unless-stopped \
              -p 8000:8000 \
              -v $INSTALL_DIR/data/chromadb:/chroma/chroma \
              ghcr.io/chroma-core/chroma:latest
        fi
    else
        echo "Warning: ChromaDB data directory not found in backup"
    fi
fi

# Set permissions
echo "Setting permissions..."
chown -R intellirouter:intellirouter $INSTALL_DIR

# Start services
if [[ "$RESTART_SERVICES" = true ]]; then
    echo "Starting IntelliRouter services..."
    systemctl start intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer || true
fi

# Clean up
echo "Cleaning up..."
if [[ "$BACKUP_FILE" == *.tar.gz ]]; then
    rm -rf $TEMP_DIR
fi

echo "=== Restore Complete ==="
echo ""
echo "IntelliRouter has been restored from backup"
echo ""
echo "To check service status:"
echo "  sudo systemctl status intellirouter-*"
echo ""