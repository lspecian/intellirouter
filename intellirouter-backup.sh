#!/bin/bash
# IntelliRouter Backup Script
# This script creates a backup of IntelliRouter configuration and data

set -e

# Default values
INSTALL_DIR="/opt/intellirouter"
BACKUP_DIR="/opt/intellirouter/backups/$(date +%Y-%m-%d)"
BACKUP_REDIS=true
BACKUP_CHROMADB=true
COMPRESS=true
ROTATE=true
ROTATION_DAYS=30
REMOTE_BACKUP=false
REMOTE_HOST=""
REMOTE_PATH=""
REMOTE_USER=""

# Display help message
function show_help {
    echo "IntelliRouter Backup Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -d, --install-dir DIR      Installation directory (default: /opt/intellirouter)"
    echo "  -b, --backup-dir DIR       Backup directory (default: /opt/intellirouter/backups/YYYY-MM-DD)"
    echo "  --no-redis                 Don't backup Redis data"
    echo "  --no-chromadb              Don't backup ChromaDB data"
    echo "  --no-compress              Don't compress the backup"
    echo "  --no-rotate                Don't rotate old backups"
    echo "  -r, --rotation-days DAYS   Number of days to keep backups (default: 30)"
    echo "  --remote-backup            Enable remote backup"
    echo "  --remote-host HOST         Remote host for backup"
    echo "  --remote-path PATH         Remote path for backup"
    echo "  --remote-user USER         Remote user for backup"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 --install-dir /opt/intellirouter --rotation-days 14"
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
        -b|--backup-dir)
            BACKUP_DIR="$2"
            shift
            shift
            ;;
        --no-redis)
            BACKUP_REDIS=false
            shift
            ;;
        --no-chromadb)
            BACKUP_CHROMADB=false
            shift
            ;;
        --no-compress)
            COMPRESS=false
            shift
            ;;
        --no-rotate)
            ROTATE=false
            shift
            ;;
        -r|--rotation-days)
            ROTATION_DAYS="$2"
            shift
            shift
            ;;
        --remote-backup)
            REMOTE_BACKUP=true
            shift
            ;;
        --remote-host)
            REMOTE_HOST="$2"
            shift
            shift
            ;;
        --remote-path)
            REMOTE_PATH="$2"
            shift
            shift
            ;;
        --remote-user)
            REMOTE_USER="$2"
            shift
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

# Validate remote backup settings
if [[ "$REMOTE_BACKUP" = true ]]; then
    if [[ -z "$REMOTE_HOST" || -z "$REMOTE_PATH" || -z "$REMOTE_USER" ]]; then
        echo "Error: Remote backup requires --remote-host, --remote-path, and --remote-user"
        exit 1
    fi
fi

echo "=== IntelliRouter Backup ==="
echo "Installation directory: $INSTALL_DIR"
echo "Backup directory: $BACKUP_DIR"
echo "Backup Redis: $BACKUP_REDIS"
echo "Backup ChromaDB: $BACKUP_CHROMADB"
echo "Compress backup: $COMPRESS"
echo "Rotate backups: $ROTATE"
echo "Rotation days: $ROTATION_DAYS"
echo "Remote backup: $REMOTE_BACKUP"
if [[ "$REMOTE_BACKUP" = true ]]; then
    echo "Remote host: $REMOTE_HOST"
    echo "Remote path: $REMOTE_PATH"
    echo "Remote user: $REMOTE_USER"
fi
echo ""

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup configuration
echo "Backing up configuration..."
cp -r $INSTALL_DIR/config $BACKUP_DIR/config
cp -r $INSTALL_DIR/env $BACKUP_DIR/env

# Backup data
echo "Backing up data..."
cp -r $INSTALL_DIR/data $BACKUP_DIR/data

# Backup Redis data
if [[ "$BACKUP_REDIS" = true ]]; then
    echo "Backing up Redis data..."
    if systemctl is-active --quiet redis-server; then
        redis-cli save
        cp /var/lib/redis/dump.rdb $BACKUP_DIR/redis-dump.rdb
    else
        echo "Warning: Redis server is not running, skipping Redis backup"
    fi
fi

# Backup ChromaDB data
if [[ "$BACKUP_CHROMADB" = true ]]; then
    echo "Backing up ChromaDB data..."
    if docker ps | grep -q chromadb; then
        # Create a temporary directory for ChromaDB backup
        mkdir -p $BACKUP_DIR/chromadb
        
        # Stop ChromaDB container to ensure data consistency
        echo "Stopping ChromaDB container..."
        docker stop chromadb
        
        # Copy ChromaDB data
        if [[ -d "$INSTALL_DIR/data/chromadb" ]]; then
            cp -r $INSTALL_DIR/data/chromadb/* $BACKUP_DIR/chromadb/
        fi
        
        # Restart ChromaDB container
        echo "Restarting ChromaDB container..."
        docker start chromadb
    else
        echo "Warning: ChromaDB container is not running, skipping ChromaDB backup"
    fi
fi

# Compress backup
if [[ "$COMPRESS" = true ]]; then
    echo "Compressing backup..."
    tar -czf $BACKUP_DIR.tar.gz -C $(dirname $BACKUP_DIR) $(basename $BACKUP_DIR)
    rm -rf $BACKUP_DIR
    BACKUP_FILE=$BACKUP_DIR.tar.gz
else
    BACKUP_FILE=$BACKUP_DIR
fi

# Copy to remote host
if [[ "$REMOTE_BACKUP" = true ]]; then
    echo "Copying backup to remote host..."
    if [[ "$COMPRESS" = true ]]; then
        scp $BACKUP_FILE $REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH
    else
        rsync -avz $BACKUP_FILE/ $REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH
    fi
fi

# Rotate backups
if [[ "$ROTATE" = true ]]; then
    echo "Rotating old backups..."
    if [[ "$COMPRESS" = true ]]; then
        find $(dirname $BACKUP_DIR) -name "*.tar.gz" -type f -mtime +$ROTATION_DAYS -delete
    else
        find $(dirname $BACKUP_DIR) -type d -mtime +$ROTATION_DAYS -exec rm -rf {} \;
    fi
fi

echo "=== Backup Complete ==="
echo "Backup saved to: $BACKUP_FILE"
if [[ "$REMOTE_BACKUP" = true ]]; then
    echo "Remote backup saved to: $REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH"
fi
echo ""