#!/bin/bash
# Git Operations Management Script
# Handles commits, pushes, conflict resolution, and rollbacks with detailed logging

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="${PROJECT_ROOT}/logs/git-operations-$(date +%Y%m%d-%H%M%S).log"
COMMIT_LOG="${PROJECT_ROOT}/logs/commit-history-$(date +%Y%m%d).log"
CHANGE_TRACKING_FILE="${PROJECT_ROOT}/logs/change-tracking-$(date +%Y%m%d).json"

# Git configuration
GIT_AUTHOR="${GIT_AUTHOR:-CI Bot}"
GIT_EMAIL="${GIT_EMAIL:-ci@rust-ai-ide.dev}"
BRANCH="${BRANCH:-main}"
REMOTE="${REMOTE:-origin}"
NOTIFICATION_WEBHOOK="${NOTIFICATION_WEBHOOK:-}"

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${LOG_FILE}"
}

# Notification function
notify_webhook() {
    local payload="$1"
    local webhook="${NOTIFICATION_WEBHOOK}"
    if [[ -n "${webhook}" ]]; then
        curl -s -X POST "${webhook}" \
            -H 'Content-type: application/json' \
            --data "${payload}" || log "WARN" "Failed to send webhook notification"
    fi
}

# Error handling
error_exit() {
    local message="$1"
    log "ERROR" "${message}"
    notify_webhook "{\"status\":\"failed\",\"operation\":\"git\",\"message\":\"${message}\",\"timestamp\":\"$(date -Iseconds)\"}"
    exit 1
}

# Ensure we're in the project root
cd "${PROJECT_ROOT}" || error_exit "Cannot change to project root directory"

# Create logs directory if it doesn't exist
mkdir -p "$(dirname "${LOG_FILE}")"

# Function to setup git configuration
setup_git_config() {
    log "INFO" "Setting up git configuration"
    git config user.name "${GIT_AUTHOR}"
    git config user.email "${GIT_EMAIL}"
    git config core.autocrlf false
    git config push.default simple
}

# Function to get detailed change information
get_change_details() {
    local since="${1:-HEAD~1}"

    log "INFO" "Analyzing changes since ${since}"

    # Get file changes
    local changed_files
    changed_files=$(git diff --name-status "${since}" | awk '{print "{\"file\":\""$2"\",\"status\":\""$1"\"},"}')

    # Get commit details
    local commit_info
    commit_info=$(git show --format='{"commit":"%H","author":"%an","email":"%ae","date":"%ai","message":"%s"}' --no-patch "${since}")

    # Get statistics
    local stats
    stats=$(git diff --stat "${since}" | tail -1)

    # Create change tracking JSON
    cat > "${CHANGE_TRACKING_FILE}" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "since_commit": "${since}",
    "current_commit": "$(git rev-parse HEAD)",
    "branch": "${BRANCH}",
    "changes": {
        "files": [${changed_files%,}],
        "stats": "${stats}",
        "commit_info": ${commit_info}
    },
    "metadata": {
        "rust_files": $(git diff --name-only "${since}" | grep '\.rs$' | wc -l),
        "typescript_files": $(git diff --name-only "${since}" | grep '\.ts\|\.tsx$' | wc -l),
        "config_files": $(git diff --name-only "${since}" | grep '\.toml\|\.json\|\.yml\|\.yaml$' | wc -l)
    }
}
EOF

    log "INFO" "Change details saved to ${CHANGE_TRACKING_FILE}"
}

# Function to create detailed commit message
create_commit_message() {
    local commit_type="$1"
    local description="$2"
    local extra_info="${3:-}"

    # Analyze changes for commit message
    local rust_changes=$(git diff --cached --name-only | grep '\.rs$' | wc -l)
    local ts_changes=$(git diff --cached --name-only | grep '\.ts\|\.tsx$' | wc -l)
    local config_changes=$(git diff --cached --name-only | grep '\.toml\|\.json\|\.yml\|\.yaml$' | wc -l)

    # Build commit message
    local commit_message="${commit_type}: ${description}

Changes:
- Rust files: ${rust_changes}
- TypeScript files: ${ts_changes}
- Config files: ${config_changes}"

    if [[ -n "${extra_info}" ]]; then
        commit_message="${commit_message}

Details: ${extra_info}"
    fi

    # Add review information if available
    if [[ -n "${REVIEWER_EMAIL:-}" ]]; then
        commit_message="${commit_message}

Reviewed-by: ${REVIEWER_EMAIL}"
    fi

    echo "${commit_message}"
}

# Function to commit changes with detailed logging
commit_changes() {
    local commit_type="$1"
    local description="$2"
    local extra_info="${3:-}"

    log "INFO" "Committing changes: ${commit_type} - ${description}"

    # Check if there are changes to commit
    if ! git diff --cached --quiet; then
        local commit_message
        commit_message=$(create_commit_message "${commit_type}" "${description}" "${extra_info}")

        # Create commit
        echo "${commit_message}" | git commit -F -

        local commit_hash
        commit_hash=$(git rev-parse HEAD)

        log "INFO" "Changes committed: ${commit_hash}"

        # Log to commit history
        echo "$(date '+%Y-%m-%d %H:%M:%S') | ${commit_hash} | ${commit_type} | ${description}" >> "${COMMIT_LOG}"

        # Get change details
        get_change_details "${commit_hash}~1"

        echo "COMMIT_HASH=${commit_hash}"
        echo "COMMIT_MESSAGE_FILE=${COMMIT_LOG}"
        echo "CHANGE_TRACKING_FILE=${CHANGE_TRACKING_FILE}"

    else
        log "INFO" "No changes to commit"
        echo "COMMIT_HASH=none"
    fi
}

# Function to handle merge conflicts
resolve_merge_conflicts() {
    log "INFO" "Attempting to resolve merge conflicts"

    # Check for conflicts
    local conflict_files
    conflict_files=$(git diff --name-only --diff-filter=U)

    if [[ -n "${conflict_files}" ]]; then
        log "WARN" "Merge conflicts detected in: ${conflict_files}"

        # For each conflict file, try automatic resolution
        echo "${conflict_files}" | while read -r file; do
            if [[ -f "${file}" ]]; then
                log "INFO" "Attempting automatic resolution for ${file}"

                # Simple strategy: prefer our changes for certain file types
                if echo "${file}" | grep -q -E '\.(lock|sum)$'; then
                    git checkout --theirs "${file}"
                    git add "${file}"
                    log "INFO" "Resolved ${file} using 'theirs' strategy"
                elif echo "${file}" | grep -q -E '\.(rs|ts|tsx|js)$'; then
                    # For code files, keep our changes
                    git checkout --ours "${file}"
                    git add "${file}"
                    log "INFO" "Resolved ${file} using 'ours' strategy"
                else
                    log "WARN" "Manual resolution required for ${file}"
                fi
            fi
        done

        # Check if all conflicts are resolved
        if git diff --name-only --diff-filter=U | grep -q .; then
            error_exit "Some merge conflicts require manual resolution"
        else
            # Complete the merge
            git commit --no-edit
            log "INFO" "Merge conflicts resolved automatically"
        fi
    else
        log "INFO" "No merge conflicts detected"
    fi
}

# Function to push changes with retry logic
push_changes() {
    local max_retries=3
    local retry_count=0

    while [[ ${retry_count} -lt ${max_retries} ]]; do
        log "INFO" "Pushing changes to ${REMOTE}/${BRANCH} (attempt $((retry_count + 1)))"

        if git push "${REMOTE}" "${BRANCH}"; then
            log "INFO" "Changes pushed successfully"
            return 0
        else
            retry_count=$((retry_count + 1))
            if [[ ${retry_count} -lt ${max_retries} ]]; then
                log "WARN" "Push failed, attempting to resolve conflicts and retry"

                # Pull with rebase to resolve conflicts
                git pull --rebase "${REMOTE}" "${BRANCH}" || true
                resolve_merge_conflicts

                # Wait before retry
                sleep $((retry_count * 5))
            fi
        fi
    done

    error_exit "Failed to push changes after ${max_retries} attempts"
}

# Function to create rollback commit
create_rollback_commit() {
    local target_commit="$1"
    local reason="$2"

    log "INFO" "Creating rollback commit to ${target_commit}"

    # Create revert commit
    git revert --no-edit "${target_commit}"

    local rollback_hash
    rollback_hash=$(git rev-parse HEAD)

    # Log rollback
    echo "$(date '+%Y-%m-%d %H:%M:%S') | ${rollback_hash} | ROLLBACK | Reverted ${target_commit} - ${reason}" >> "${COMMIT_LOG}"

    log "INFO" "Rollback commit created: ${rollback_hash}"
    echo "ROLLBACK_COMMIT=${rollback_hash}"
}

# Function to rollback to specific commit
rollback_to_commit() {
    local target_commit="$1"
    local reason="$2"

    log "INFO" "Rolling back to commit ${target_commit}"

    # Reset to target commit
    git reset --hard "${target_commit}"

    # Create a rollback marker commit
    local rollback_message="ROLLBACK: Reset to ${target_commit}

Reason: ${reason}
Original commit: ${target_commit}
Rolled back by: ${GIT_AUTHOR} <${GIT_EMAIL}>
Timestamp: $(date)"

    echo "${rollback_message}" | git commit -F -

    local rollback_hash
    rollback_hash=$(git rev-parse HEAD)

    log "INFO" "Rollback completed: ${rollback_hash}"
    echo "ROLLBACK_COMMIT=${rollback_hash}"
}

# Function to check repository status
check_repo_status() {
    log "INFO" "Checking repository status"

    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        error_exit "Not a git repository"
    fi

    # Check for uncommitted changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
        log "WARN" "Uncommitted changes detected"
        echo "UNCOMMITTED_CHANGES=true"
    else
        echo "UNCOMMITTED_CHANGES=false"
    fi

    # Check if we're ahead/behind remote
    local ahead_behind
    ahead_behind=$(git rev-list --count --left-right "${REMOTE}/${BRANCH}"..."${BRANCH}" 2>/dev/null || echo "unknown")
    echo "GIT_STATUS=${ahead_behind}"
}

# Main execution functions
main_commit() {
    local commit_type="$1"
    local description="$2"
    local extra_info="${3:-}"

    setup_git_config
    check_repo_status

    # Stage all changes if none staged
    if git diff --cached --quiet; then
        git add .
        log "INFO" "Staged all changes"
    fi

    commit_changes "${commit_type}" "${description}" "${extra_info}"
}

main_push() {
    setup_git_config
    check_repo_status
    push_changes
}

main_rollback() {
    local target_commit="$1"
    local reason="$2"

    setup_git_config
    rollback_to_commit "${target_commit}" "${reason}"
}

# Command dispatcher
case "${1:-}" in
    "commit")
        shift
        main_commit "$@"
        ;;
    "push")
        main_push
        ;;
    "rollback")
        shift
        main_rollback "$@"
        ;;
    "status")
        check_repo_status
        ;;
    *)
        echo "Usage: $0 {commit|push|rollback|status} [args...]"
        echo "  commit <type> <description> [extra_info]  - Commit changes"
        echo "  push                                      - Push changes with conflict resolution"
        echo "  rollback <commit> <reason>               - Rollback to specific commit"
        echo "  status                                    - Check repository status"
        exit 1
        ;;
esac

# Success notification
notify_webhook "{\"status\":\"completed\",\"operation\":\"git\",\"timestamp\":\"$(date -Iseconds)\"}"