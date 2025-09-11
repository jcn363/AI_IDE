#!/bin/bash

# OWASP ZAP Web Component Security Scanning Script
# Automated vulnerability scanning for Rust AI IDE web frontend
# Author: Security Team
# Version: 1.0.0

set -euo pipefail

# Constants
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/web"
SECURITY_LOG="${PROJECT_ROOT}/zap-security.log"
REPORT_DIR="${PROJECT_ROOT}/security-reports/zap"
START_TIME=$(date +%s)

# Default configuration
ZAP_PORT=8080
WEB_PORT=3000
ZAP_API_KEY=""
ZAP_PATH=""

# Create report directory
mkdir -p "${REPORT_DIR}"

# Logging functions
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*" | tee -a "${SECURITY_LOG}"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "${SECURITY_LOG}" >&2
}

log_warning() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $*" | tee -a "${SECURITY_LOG}"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $*" | tee -a "${SECURITY_LOG}"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

OWASP ZAP security scanning for Rust AI IDE web components.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -p, --web-port PORT     Web server port (default: 3000)
    -z, --zap-port PORT     ZAP proxy port (default: 8080)
    -k, --api-key KEY       ZAP API key (from environment or secrets)
    --zap-path PATH         Path to ZAP installation
    --report-dir DIR        Output directory for reports (default: security-reports/zap)
    --scheduled             Run in scheduled mode (non-interactive)
    --baseline              Run baseline scan (passive scanning)
    --full-scan             Run full active scan (may impact performance)

EXAMPLES:
    $0 --baseline
    $0 --full-scan --web-port 3001
    $0 --scheduled --api-key "${ZAP_API_KEY}"

EOF
}

# Parse command line arguments
VERBOSE=false
SCHEDULED=false
BASELINE_SCAN=true
FULL_SCAN=false
WEB_PORT_ARG=${WEB_PORT}

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -p|--web-port)
            WEB_PORT_ARG="$2"
            shift 2
            ;;
        -z|--zap-port)
            ZAP_PORT="$2"
            shift 2
            ;;
        -k|--api-key)
            ZAP_API_KEY="$2"
            shift 2
            ;;
        --zap-path)
            ZAP_PATH="$2"
            shift 2
            ;;
        --report-dir)
            REPORT_DIR="$2"
            shift 2
            ;;
        --scheduled)
            SCHEDULED=true
            shift
            ;;
        --baseline)
            BASELINE_SCAN=true
            FULL_SCAN=false
            shift
            ;;
        --full-scan)
            FULL_SCAN=true
            BASELINE_SCAN=false
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Function to check system requirements
check_zap_requirements() {
    log_info "Checking ZAP requirements..."

    # Check if ZAP is available
    if [[ -n "${ZAP_PATH}" ]]; then
        if [[ ! -f "${ZAP_PATH}/zap.sh" ]]; then
            log_error "ZAP not found at specified path: ${ZAP_PATH}"
            return 1
        fi
    else
        # Try to find ZAP in common locations
        ZAP_PATHS=(
            "/opt/zaproxy"
            "/usr/local/zaproxy"
            "/usr/share/zaproxy"
            "${HOME}/ZAP"
            "/Applications/ZAP.app/Contents/Java"
        )

        for path in "${ZAP_PATHS[@]}"; do
            if [[ -f "${path}/zap.sh" ]]; then
                ZAP_PATH="${path}"
                log_info "Found ZAP at: ${ZAP_PATH}"
                break
            fi
        done

        if [[ -z "${ZAP_PATH}" ]]; then
            log_error "OWASP ZAP not found. Please install ZAP or specify --zap-path"
            log_info "Installation options:"
            log_info "  - Download from: https://www.zaproxy.org/download/"
            log_info "  - Or use docker: docker pull owasp/zap2docker-stable"
            return 1
        fi
    fi

    # Check Node.js and npm
    if ! command -v node >/dev/null 2>&1; then
        log_error "Node.js not found"
        return 1
    fi

    if ! command -v npm >/dev/null 2>&1; then
        log_error "npm not found"
        return 1
    fi

    # Check web directory
    if [[ ! -f "${WEB_DIR}/package.json" ]]; then
        log_error "Web directory not found or invalid: ${WEB_DIR}"
        return 1
    fi
}

# Function to install ZAP if needed
install_zap() {
    if [[ "${SCHEDULED}" == true ]]; then
        return 0
    fi

    log_info "Installing/configuring OWASP ZAP..."

    # Try Docker installation first (easier for CI/CD)
    if command -v docker >/dev/null 2>&1; then
        log_info "Using Docker-based ZAP installation"
        ZAP_DOCKER=true
        return 0
    fi

    # Manual installation instructions
    log_info "Please install OWASP ZAP manually:"
    log_info "1. Download from: https://www.zaproxy.org/download/"
    log_info "2. Extract to a directory"
    log_info "3. Run with --zap-path /path/to/zap"
    return 1
}

# Function to start web server
start_web_server() {
    log_info "Starting web development server..."

    cd "${WEB_DIR}"

    # Start the dev server in background
    npm run dev > "${REPORT_DIR}/web-server.log" 2>&1 &
    WEB_PID=$!

    log_info "Web server started with PID: ${WEB_PID}"

    # Wait for server to be ready
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -s -f "http://localhost:${WEB_PORT_ARG}" >/dev/null 2>&1; then
            log_success "Web server is ready on port ${WEB_PORT_ARG}"
            return 0
        fi

        log_info "Waiting for web server to start (attempt ${attempt}/${max_attempts})..."
        sleep 2
        ((attempt++))
    done

    log_error "Web server failed to start within timeout"
    kill "${WEB_PID}" 2>/dev/null || true
    return 1
}

# Function to stop web server
stop_web_server() {
    if [[ -n "${WEB_PID:-}" ]]; then
        log_info "Stopping web server (PID: ${WEB_PID})..."
        kill "${WEB_PID}" 2>/dev/null || true
        wait "${WEB_PID}" 2>/dev/null || true
        log_success "Web server stopped"
    fi
}

# Function to start ZAP
start_zap() {
    log_info "Starting OWASP ZAP..."

    if [[ "${ZAP_DOCKER:-false}" == true ]]; then
        # Docker-based ZAP
        ZAP_CONTAINER=$(docker run -d -p "${ZAP_PORT}:8080" \
            -v "${REPORT_DIR}:/zap/reports" \
            owasp/zap2docker-stable zap.sh -daemon -port 8080 \
            -config api.key="${ZAP_API_KEY}")

        log_info "ZAP container started: ${ZAP_CONTAINER}"

        # Wait for ZAP to be ready
        local max_attempts=60
        local attempt=1

        while [[ $attempt -le $max_attempts ]]; do
            if curl -s "http://localhost:${ZAP_PORT}" >/dev/null 2>&1; then
                log_success "ZAP is ready on port ${ZAP_PORT}"
                return 0
            fi

            log_info "Waiting for ZAP to start (attempt ${attempt}/${max_attempts})..."
            sleep 2
            ((attempt++))
        done

        log_error "ZAP failed to start within timeout"
        return 1
    else
        # Local ZAP installation
        "${ZAP_PATH}/zap.sh" -daemon -port "${ZAP_PORT}" \
            -config api.key="${ZAP_API_KEY}" \
            > "${REPORT_DIR}/zap.log" 2>&1 &
        ZAP_PID=$!

        log_info "ZAP started with PID: ${ZAP_PID}"

        # Wait for ZAP to be ready
        local max_attempts=60
        local attempt=1

        while [[ $attempt -le $max_attempts ]]; do
            if curl -s "http://localhost:${ZAP_PORT}" >/dev/null 2>&1; then
                log_success "ZAP is ready on port ${ZAP_PORT}"
                return 0
            fi

            log_info "Waiting for ZAP to start (attempt ${attempt}/${max_attempts})..."
            sleep 2
            ((attempt++))
        done

        log_error "ZAP failed to start within timeout"
        return 1
    fi
}

# Function to stop ZAP
stop_zap() {
    if [[ "${ZAP_DOCKER:-false}" == true ]] && [[ -n "${ZAP_CONTAINER:-}" ]]; then
        log_info "Stopping ZAP container..."
        docker stop "${ZAP_CONTAINER}" >/dev/null 2>&1 || true
        docker rm "${ZAP_CONTAINER}" >/dev/null 2>&1 || true
        log_success "ZAP container stopped"
    elif [[ -n "${ZAP_PID:-}" ]]; then
        log_info "Stopping ZAP (PID: ${ZAP_PID})..."
        kill "${ZAP_PID}" 2>/dev/null || true
        wait "${ZAP_PID}" 2>/dev/null || true
        log_success "ZAP stopped"
    fi
}

# Function to run ZAP scan
run_zap_scan() {
    log_info "Running ZAP security scan..."

    local target_url="http://localhost:${WEB_PORT_ARG}"
    local zap_api_url="http://localhost:${ZAP_PORT}"
    local scan_report="${REPORT_DIR}/zap-scan-report.json"
    local html_report="${REPORT_DIR}/zap-scan-report.html"

    # Configure ZAP context and target
    log_info "Configuring ZAP scan context..."

    if [[ "${ZAP_DOCKER:-false}" == true ]]; then
        # Docker-based scan
        if [[ "${BASELINE_SCAN}" == true ]]; then
            log_info "Running baseline scan..."
            docker exec "${ZAP_CONTAINER}" zap-baseline.py \
                -t "${target_url}" \
                -r "${html_report}" \
                -x "${scan_report}" \
                > "${REPORT_DIR}/zap-baseline.log" 2>&1
        else
            log_info "Running full active scan..."
            docker exec "${ZAP_CONTAINER}" zap-full-scan.py \
                -t "${target_url}" \
                -r "${html_report}" \
                -x "${scan_report}" \
                > "${REPORT_DIR}/zap-full.log" 2>&1
        fi
    else
        # Local ZAP scan using API
        log_info "Using ZAP API for scanning..."

        # Spider the target
        curl -s "${zap_api_url}/JSON/spider/action/scan/" \
            -d "url=${target_url}" \
            -d "apikey=${ZAP_API_KEY}" >/dev/null

        # Wait for spider to complete
        local spider_status="running"
        while [[ "${spider_status}" != "0" ]]; do
            sleep 5
            spider_status=$(curl -s "${zap_api_url}/JSON/spider/view/status/" \
                -d "apikey=${ZAP_API_KEY}" | jq -r '.status' 2>/dev/null || echo "100")
            log_info "Spider progress: ${spider_status}%"
        done

        # Run active scan if requested
        if [[ "${FULL_SCAN}" == true ]]; then
            log_info "Running active scan..."
            curl -s "${zap_api_url}/JSON/ascan/action/scan/" \
                -d "url=${target_url}" \
                -d "apikey=${ZAP_API_KEY}" >/dev/null

            # Wait for active scan to complete
            local ascan_status="running"
            while [[ "${ascan_status}" != "100" ]]; do
                sleep 10
                ascan_status=$(curl -s "${zap_api_url}/JSON/ascan/view/status/" \
                    -d "apikey=${ZAP_API_KEY}" | jq -r '.status' 2>/dev/null || echo "100")
                log_info "Active scan progress: ${ascan_status}%"
            done
        fi

        # Generate reports
        log_info "Generating ZAP reports..."

        # JSON report
        curl -s "${zap_api_url}/JSON/core/view/alerts/" \
            -d "apikey=${ZAP_API_KEY}" \
            > "${scan_report}"

        # HTML report
        curl -s "${zap_api_url}/OTHER/core/other/htmlreport/" \
            -d "apikey=${ZAP_API_KEY}" \
            > "${html_report}"
    fi

    # Analyze results
    analyze_zap_results "${scan_report}"
}

# Function to analyze ZAP results
analyze_zap_results() {
    local report_file="$1"
    log_info "Analyzing ZAP scan results..."

    local high_risk=$(jq '[.alerts[] | select(.risk == "High")] | length' "${report_file}" 2>/dev/null || echo "0")
    local medium_risk=$(jq '[.alerts[] | select(.risk == "Medium")] | length' "${report_file}" 2>/dev/null || echo "0")
    local low_risk=$(jq '[.alerts[] | select(.risk == "Low")] | length' "${report_file}" 2>/dev/null || echo "0")
    local info_risk=$(jq '[.alerts[] | select(.risk == "Informational")] | length' "${report_file}" 2>/dev/null || echo "0")

    log_info "ZAP Scan Results:"
    log_info "  High Risk: ${high_risk}"
    log_info "  Medium Risk: ${medium_risk}"
    log_info "  Low Risk: ${low_risk}"
    log_info "  Informational: ${info_risk}"

    # Generate summary report
    jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg target "http://localhost:${WEB_PORT_ARG}" \
        --arg high_risk "$high_risk" \
        --arg medium_risk "$medium_risk" \
        --arg low_risk "$low_risk" \
        --arg info_risk "$info_risk" \
        --arg scan_type "$( [[ "${FULL_SCAN}" == true ]] && echo "Active" || echo "Baseline" )" \
        '{
            timestamp: $timestamp,
            tool: "OWASP ZAP",
            target: $target,
            scan_type: $scan_type,
            vulnerabilities: {
                high: ($high_risk | tonumber),
                medium: ($medium_risk | tonumber),
                low: ($low_risk | tonumber),
                informational: ($info_risk | tonumber)
            },
            status: (if ($high_risk == "0") then "PASSED" elif ($high_risk | tonumber) > 0 then "FAILED" else "WARNING" end)
        }' > "${REPORT_DIR}/zap-summary-report.json"

    if [[ "${high_risk}" -gt 0 ]]; then
        log_error "High-risk vulnerabilities found: ${high_risk}"
        return 1
    elif [[ "${medium_risk}" -gt 0 ]]; then
        log_warning "Medium-risk vulnerabilities found: ${medium_risk}"
        return 0
    else
        log_success "No high-risk vulnerabilities found"
        return 0
    fi
}

# Function to setup scheduled scanning
setup_scheduled_scan() {
    log_info "Setting up scheduled ZAP scanning..."

    local cron_schedule="0 2 * * *"  # Daily at 2 AM
    local cron_command="${SCRIPT_DIR}/zap-web-scan.sh --scheduled --baseline"

    # Add to crontab
    if ! crontab -l 2>/dev/null | grep -q "zap-web-scan.sh"; then
        (crontab -l 2>/dev/null; echo "${cron_schedule} ${cron_command}") | crontab -
        log_success "Scheduled ZAP scan added to crontab"
    else
        log_info "Scheduled ZAP scan already exists in crontab"
    fi

    # Generate schedule report
    jq -n \
        --arg schedule "$cron_schedule" \
        --arg command "$cron_command" \
        --arg setup_time "$(date -Iseconds)" \
        '{
            setup_time: $setup_time,
            schedule: $schedule,
            command: $command,
            status: "ACTIVE"
        }' > "${REPORT_DIR}/zap-schedule.json"
}

# Trap to ensure cleanup on exit
trap 'stop_web_server; stop_zap; log_info "ZAP scan completed (exit code: $?)"; [[ -d "${REPORT_DIR}" ]] && echo "Reports available in: ${REPORT_DIR}"' EXIT

# Main function
main() {
    log_info "Starting OWASP ZAP web component security scan"
    log_info "Log file: ${SECURITY_LOG}"
    log_info "Report directory: ${REPORT_DIR}"

    mkdir -p "${REPORT_DIR}"

    check_zap_requirements
    install_zap

    local exit_code=0

    # Start services
    start_web_server || exit_code=$((exit_code + 1))
    start_zap || exit_code=$((exit_code + 1))

    if [[ $exit_code -eq 0 ]]; then
        # Run the scan
        run_zap_scan || exit_code=$((exit_code + 1))

        # Setup scheduled scanning if requested
        if [[ "${SCHEDULED}" == false ]]; then
            setup_scheduled_scan
        fi
    fi

    local end_time=$(date +%s)
    log_info "ZAP scan completed in $((end_time - START_TIME)) seconds"

    return $exit_code
}

# Run main function
main "$@"