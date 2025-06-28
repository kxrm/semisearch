#!/bin/bash
# Simple backup script for personal documents
# Usage: ./backup.sh [source_directory] [destination_directory]

# Default directories
SOURCE_DIR=${1:-"$HOME/Documents"}
DEST_DIR=${2:-"$HOME/Backups"}
BACKUP_NAME="backup_$(date +%Y%m%d_%H%M%S).tar.gz"

# Check if source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Source directory $SOURCE_DIR does not exist."
    exit 1
fi

# Create destination directory if it doesn't exist
if [ ! -d "$DEST_DIR" ]; then
    echo "Creating destination directory $DEST_DIR..."
    mkdir -p "$DEST_DIR"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to create destination directory $DEST_DIR."
        exit 1
    fi
fi

# Display backup information
echo "Backup started at $(date)"
echo "Source: $SOURCE_DIR"
echo "Destination: $DEST_DIR/$BACKUP_NAME"

# Create backup
echo "Creating backup..."
tar -czf "$DEST_DIR/$BACKUP_NAME" -C "$(dirname "$SOURCE_DIR")" "$(basename "$SOURCE_DIR")" 2>/dev/null

# Check if backup was successful
if [ $? -eq 0 ]; then
    echo "Backup completed successfully!"
    echo "Backup size: $(du -h "$DEST_DIR/$BACKUP_NAME" | cut -f1)"
    echo "Backup location: $DEST_DIR/$BACKUP_NAME"
else
    echo "Error: Backup failed."
    exit 1
fi

# Clean up old backups (keep last 5)
echo "Cleaning up old backups..."
ls -t "$DEST_DIR"/backup_*.tar.gz 2>/dev/null | tail -n +6 | xargs -r rm
echo "Cleanup completed."

echo "Backup process completed at $(date)"
exit 0 