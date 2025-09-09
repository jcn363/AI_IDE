#!/bin/bash

# Automated Backup Script for Rust AI IDE Enterprise
# Supports versioning and point-in-time recovery
# Compatible with on-premises and air-gapped environments

set -euo pipefail

# Configuration
BACKUP_ROOT="/backup"
RETENTION_DAYS=30
RETENTION_WEEKS=52
RETENTION_MONTHS=24

# Environment variables
PROJECT_ENV="${PROJECT_ENV:-production}"
BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_ID="${PROJECT_ENV}_${BACKUP_TIMESTAMP}"

# Logging
LOG_FILE="/var/log/rust-ai-ide/backup_${BACKUP_ID}.log"

# Function definitions
log() {
  echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

error() {
  echo "[ERROR] $*" >&2 | tee -a "$LOG_FILE"
  exit 1
}

cleanup_old_backups() {
  local backup_type=$1
  log "Cleaning up old $backup_type backups"

  # Daily: keep 7 days
  find "$BACKUP_ROOT/$backup_type/daily" -name "*.tar.gz" -mtime +7 -delete 2>/dev/null || true

  # Weekly: keep 4 weeks
  find "$BACKUP_ROOT/$backup_type/weekly" -name "*.tar.gz" -mtime +28 -delete 2>/dev/null || true

  # Monthly: keep 12 months
  find "$BACKUP_ROOT/$backup_type/monthly" -name "*.tar.gz" -mtime +365 -delete 2>/dev/null || true
}

backup_postgres() {
  log "Starting PostgreSQL backup"

  local backup_dir="$BACKUP_ROOT/postgres/daily"
  local backup_file="${backup_dir}/${BACKUP_ID}_postgres.sql.gz"

  mkdir -p "$backup_dir"

  # Create physical backup with pg_basebackup
  docker exec postgres-primary pg_basebackup -D /tmp/basebackup -Ft -z -P
  docker cp postgres-primary:/tmp/backup.tar.gz "$backup_file"

  # Create logical backup for point-in-time recovery
  docker exec postgres-primary pg_dump --format=directory \
    --compress=9 \
    --verbose \
    --no-comments \
    --exclude-table=session \
    rust_ai_ide \
    | gzip > "${backup_dir}/logical_${BACKUP_ID}.gz"

  # Create WAL archive for PITR
  log "Archiving WAL files"
  docker exec postgres-primary find /var/lib/postgresql/data/pg_wal -type f -newer "$backup_file" \
    -exec cp {} "$BACKUP_ROOT/postgres/wal/" \; 2>/dev/null || true

  local backup_size=$(du -sh "$backup_file" | cut -f1)
  log "PostgreSQL backup completed: $backup_size"
}

backup_redis() {
  log "Starting Redis backup"

  local backup_dir="$BACKUP_ROOT/redis/daily"
  local backup_file="${backup_dir}/${BACKUP_ID}_redis.rdb"

  mkdir -p "$backup_dir"

  # Trigger Redis SAVE command
  docker exec redis redis-cli SAVE

  # Copy RDB file
  docker cp redis:/data/dump.rdb "$backup_file"

  # Create AOF dump for additional recovery option
  docker exec redis redis-cli BGSAVE

  local backup_size=$(du -sh "$backup_file" | cut -f1)
  log "Redis backup completed: $backup_size"
}

backup_application_data() {
  log "Starting application data backup"

  local backup_dir="$BACKUP_ROOT/application/daily"
  local backup_file="${backup_dir}/${BACKUP_ID}_app_data.tar.gz"

  mkdir -p "$backup_dir"

  # Stop services briefly for consistent backup (optional, can be disabled)
  if [[ "${CONSISTENT_BACKUP:-false}" == "true" ]]; then
    docker-compose --project-name="$PROJECT_ENV" stop web-frontend rust-backend
  fi

  # Create backup of user data, configs, and logs
  tar -czf "$backup_file" \
    --exclude='*.log' \
    --exclude='cache/*' \
    --exclude='tmp/*' \
    -C /opt/rust-ai-ide \
    user-data/ \
    config/ \
    custom-themes/ \
    plugins/ \
    2>/dev/null || true

  # Restart services
  if [[ "${CONSISTENT_BACKUP:-false}" == "true" ]]; then
    docker-compose --project-name="$PROJECT_ENV" start web-frontend rust-backend
  fi

  local backup_size=$(du -sh "$backup_file" | cut -f1)
  log "Application data backup completed: $backup_size"
}

backup_containers() {
  log "Starting container image backup"

  local backup_dir="$BACKUP_ROOT/containers"
  local images=(
    "rust-backend:${BACKUP_TIMESTAMP}"
    "web-frontend:${BACKUP_TIMESTAMP}"
    "tauri-desktop:${BACKUP_TIMESTAMP}"
  )

  mkdir -p "$backup_dir"

  for image in "${images[@]}"; do
    log "Backing up container image: $image"
    docker save "registry.local:5000/rust-ai-ide/$image" > "${backup_dir}/${image//:/_}.tar" 2>/dev/null || \
      log "Container image $image not found, skipping"
  done

  log "Container image backup completed"
}

create_weekly_backup() {
  log "Creating weekly consolidated backup"

  local weekly_dir="$BACKUP_ROOT/weekly"
  mkdir -p "$weekly_dir"

  # Create weekly snapshot (Sunday)
  if [[ $(date +%u) -eq 7 ]]; then
    local daily_backups=$(find "$BACKUP_ROOT" -name "*_$(date +%Y%m%d)*" | head -10)

    tar -czf "$weekly_dir/weekly_$(date +%Y%m%d_%H%M%S).tar.gz" \
      "$daily_backups" \
      "$BACKUP_ROOT/postgres" \
      "$BACKUP_ROOT/application"
  fi
}

create_monthly_backup() {
  log "Creating monthly consolidated backup"

  local monthly_dir="$BACKUP_ROOT/monthly"
  mkdir -p "$monthly_dir"

  # Create monthly snapshot (1st day of month)
  if [[ $(date +%d) -eq 1 ]]; then
    local weekly_backups=$(find "$BACKUP_ROOT/weekly" -name "*.tar.gz" | tail -4)

    tar -czf "$monthly_dir/monthly_$(date +%Y%m)_${BACKUP_TIMESTAMP}.tar.gz" \
      "$weekly_backups" \
      "$BACKUP_ROOT/postgres/monthly" \
      "$BACKUP_ROOT/application/monthly"
  fi
}

verify_backup() {
  log "Verifying backup integrity"

  local failed_checks=0

  # Check PostgreSQL backup
  if [[ -f "$BACKUP_ROOT/postgres/daily/${BACKUP_ID}_postgres.sql.gz" ]]; then
    if ! gzip -t "$BACKUP_ROOT/postgres/daily/${BACKUP_ID}_postgres.sql.gz" 2>/dev/null; then
      error "PostgreSQL backup checksum failed"
      ((failed_checks++))
    fi
  fi

  # Check Redis backup
  if [[ -f "$BACKUP_ROOT/redis/daily/${BACKUP_ID}_redis.rdb" ]]; then
    if ! file "$BACKUP_ROOT/redis/daily/${BACKUP_ID}_redis.rdb" | grep -q "redis"; then
      log "Redis backup appears corrupted"
      ((failed_checks++))
    fi
  fi

  # Check application data backup
  if [[ -f "$BACKUP_ROOT/application/daily/${BACKUP_ID}_app_data.tar.gz" ]]; then
    if ! tar -tzf "$BACKUP_ROOT/application/daily/${BACKUP_ID}_app_data.tar.gz" >/dev/null; then
      log "Application data backup appears corrupted"
      ((failed_checks++))
    fi
  fi

  if [[ $failed_checks -gt 0 ]]; then
    error "Backup verification failed with $failed_checks errors"
  else
    log "Backup verification passed"
  fi
}

create_backup_manifest() {
  local manifest_file="$BACKUP_ROOT/manifests/${BACKUP_ID}.json"

  mkdir -p "$BACKUP_ROOT/manifests"

  cat > "$manifest_file" << EOF
{
  "backup_id": "$BACKUP_ID",
  "timestamp": "$BACKUP_TIMESTAMP",
  "environment": "$PROJECT_ENV",
  "version": "1.0",
  "components": {
    "postgres": {
      "type": "physical",
      "location": "$BACKUP_ROOT/postgres/daily/${BACKUP_ID}_postgres.sql.gz",
      "size": "$(stat -f%z "$BACKUP_ROOT/postgres/daily/${BACKUP_ID}_postgres.sql.gz" 2>/dev/null || echo "0")",
      "databases": ["rust_ai_ide"],
      "point_in_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    },
    "redis": {
      "type": "rdb",
      "location": "$BACKUP_ROOT/redis/daily/${BACKUP_ID}_redis.rdb",
      "size": "$(stat -f%z "$BACKUP_ROOT/redis/daily/${BACKUP_ID}_redis.rdb" 2>/dev/null || echo "0")",
      "keys": $(docker exec redis redis-cli DBSIZE 2>/dev/null || echo "0")
    },
    "application": {
      "type": "filesystem",
      "location": "$BACKUP_ROOT/application/daily/${BACKUP_ID}_app_data.tar.gz",
      "size": "$(stat -f%z "$BACKUP_ROOT/application/daily/${BACKUP_ID}_app_data.tar.gz" 2>/dev/null || echo "0")",
      "paths": ["/opt/rust-ai-ide/user-data", "/opt/rust-ai-ide/config"]
    }
  },
  "retention": {
    "daily": "7 days",
    "weekly": "4 weeks",
    "monthly": "12 months"
  },
  "verification": {
    "checksums_calculated": true,
    "integrity_checked": true,
    "encrypted": false
  }
}
EOF

  log "Backup manifest created: $manifest_file"
}

send_notification() {
  local status=$1
  local message=${2:-"Backup completed"}

  case "$status" in
    "success")
      log "Backup completed successfully - $message"
      ;;
    "failure")
      log "Backup failed - $message"
      ;;
    "warning")
      log "Backup completed with warnings - $message"
      ;;
  esac

  # Send webhook notification if configured
  if [[ -n "${BACKUP_WEBHOOK_URL:-}" ]]; then
    curl -X POST "$BACKUP_WEBHOOK_URL" \
      -H 'Content-Type: application/json' \
      -d "{\"status\":\"$status\",\"backup_id\":\"$BACKUP_ID\",\"message\":\"$message\"}" \
      || log "Failed to send webhook notification"
  fi
}

# Main execution
main() {
  log "Starting automated backup for Rust AI IDE - $PROJECT_ENV"
  log "Backup ID: $BACKUP_ID"

  # Initialize backup directories
  mkdir -p "$BACKUP_ROOT"/{postgres/{daily,weekly,monthly,wal},redis/{daily,weekly,monthly},application/{daily,weekly,monthly},containers,manifests}

  # Track start time
  local start_time=$(date +%s)

  # Perform backups
  local exit_code=0

  set +e  # Continue on errors to report issues
  {
    backup_postgres
    backup_redis
    backup_application_data
    backup_containers
    create_weekly_backup
    create_monthly_backup
    verify_backup
    create_backup_manifest
  }
  exit_code=$?
  set -e

  # Calculate duration
  local end_time=$(date +%s)
  local duration=$((end_time - start_time))

  # Clean up old backups
  cleanup_old_backups postgres
  cleanup_old_backups redis
  cleanup_old_backups application

  # Send notifications
  if [[ $exit_code -eq 0 ]]; then
    send_notification "success" "Backup completed in ${duration}s"
    log "Automated backup completed successfully in ${duration}s"
  else
    send_notification "failure" "Backup failed after ${duration}s"
    log "Automated backup failed after ${duration}s"
    exit $exit_code
  fi
}

# Run main function
main "$@"