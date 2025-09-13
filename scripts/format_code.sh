#!/bin/bash

# Format Code Script for Rust AI IDE
# This script ensures consistent code formatting across the entire workspace

# Enable strict mode
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUST_TOOLCHAIN="nightly" # or "stable" if preferred
PARALLEL_JOBS=$(nproc)   # Number of parallel jobs based on CPU cores

# Function to print status messages
status() {
	echo -e "${BLUE}==>${NC} $1"
}

# Function to print success messages
success() {
	echo -e "${GREEN}✓${NC} $1"
}

# Function to print warning messages
warning() {
	echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to print error messages and exit
error() {
	echo -e "${RED}✗ Error: $1${NC}" >&2
	exit 1
}

# Function to check if a command exists
command_exists() {
	command -v "$1" >/dev/null 2>&1
}

# Function to install rustfmt if not present
install_rustfmt() {
	status "Checking for rustfmt..."
	# Check if rustfmt exists first
	if command -v rustfmt >/dev/null 2>&1; then
		success "rustfmt is already installed"
		return 0
	fi

	warning "rustfmt not found. Installing..."
	if ! rustup component add rustfmt --toolchain "${RUST_TOOLCHAIN}"; then
		error "Failed to install rustfmt. Please install it manually with: rustup component add rustfmt"
	fi

	# Verify installation
	if ! command -v rustfmt >/dev/null 2>&1; then
		error "rustfmt installation verification failed. Please install it manually with: rustup component add rustfmt"
	fi

	success "rustfmt is installed"
}

# Function to check toolchain
check_toolchain() {
	status "Checking Rust toolchain..."
	if ! rustup show active-toolchain | grep -q "${RUST_TOOLCHAIN}"; then
		warning "${RUST_TOOLCHAIN} toolchain is not active. Installing..."
		if ! rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal; then
			error "Failed to install ${RUST_TOOLCHAIN} toolchain"
		fi
		if ! rustup component add rustfmt --toolchain "${RUST_TOOLCHAIN}"; then
			error "Failed to add rustfmt component to ${RUST_TOOLCHAIN} toolchain"
		fi
	fi

	# Verify rustfmt is available
	if ! command -v rustfmt >/dev/null 2>&1; then
		error "rustfmt is not available. Please install it with: rustup component add rustfmt"
	fi

	local rustc_version
	rustc_version=$(rustc --version 2>/dev/null || echo "unknown version")
	local rustfmt_version
	rustfmt_version=$(rustfmt --version 2>/dev/null || echo "not available")
	success "Using ${rustc_version} (rustfmt ${rustfmt_version})"
}

# Function to clean up trailing whitespace
clean_whitespace() {
	status "Cleaning up trailing whitespace..."
	find . \( -path "*/target" -prune -o -path "*/.cargo" -prune -o -path "*/build" -prune \) -o -type f -name '*.rs' -exec sed -i 's/[[:space:]]*$//' {} \;
	return 0
}

# Function to check for problematic crates (kept for compatibility)
check_problematic_crates() {
	echo "" # No problematic crates with direct formatting
	return 0
}

# Function to check for Prettier and ESLint in web directory
check_prettier_and_eslint() {
	status "Checking for Prettier and ESLint in web/ directory..."
	if [[ ! -d "web" ]]; then
		warning "web/ directory not found. Skipping TypeScript/JavaScript formatting."
		return 1
	fi

	cd web
	if ! npx prettier --version >/dev/null 2>&1; then
		warning "Prettier not found in web/. Please install dependencies with: npm install"
		cd ..
		return 1
	fi

	if ! npx eslint --version >/dev/null 2>&1; then
		warning "ESLint not found in web/. Please install dependencies with: npm install"
		cd ..
		return 1
	fi

	cd ..
	success "Prettier and ESLint are available in web/ directory"
	return 0
}

# Function to format TypeScript/JavaScript files in web/ directory
format_js_ts() {
	local check_only=${1:-0}
	local js_files
	js_files=$(find web/src \( -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" -o -name "*.json" -o -name "*.css" -o -name "*.scss" -o -name "*.md" \) 2>/dev/null | wc -l | tr -d '\n')
	TOTAL_FILES=$((TOTAL_FILES + js_files))
	status "Formatting TypeScript/JavaScript files in web/ directory..."

	cd web

	# Check for large files that might cause memory issues
	local large_files
	large_files=$(find src -name "*.{js,jsx,ts,tsx}" -size +5M 2>/dev/null)
	if [[ -n ${large_files} ]]; then
	  warning "Found very large JS/TS files (>5MB) that will be skipped to prevent memory issues:"
	  echo "${large_files}" | while read -r file; do
	    warning "  - ${file} ($(du -h "${file}" | cut -f1))"
	  done
	fi

	local exit_code=0
	local node_memory="--max-old-space-size=8192"

	if [[ ${check_only} -eq 1 ]]; then
		if ! NODE_OPTIONS="${node_memory}" npx prettier --check "src/**/*.{js,jsx,ts,tsx,json,css,scss,md}" >/dev/null 2>&1; then
			warning "Prettier formatting issues found in web/ directory"
			exit_code=1
		fi

		if ! NODE_OPTIONS="${node_memory}" npx eslint --ext .js,.jsx,.ts,.tsx src/ --quiet --max-warnings 0 2>/dev/null | head -20 >/dev/null 2>&1; then
			warning "ESLint issues found in web/ directory"
			exit_code=1
		fi
	else
		# Process Prettier with memory optimization
		if ! NODE_OPTIONS="${node_memory}" timeout 300 npx prettier --write "src/**/*.{js,jsx,ts,tsx,json,css,scss,md}" 2>&1 | grep -v "(unchanged)" | tee /tmp/prettier_output; then
			warning "Prettier formatting failed or timed out in web/ directory"
			exit_code=1
			ERROR_FILES=$((ERROR_FILES + js_files))
		fi
		local changed
		changed=$(wc -l </tmp/prettier_output 2>/dev/null || echo "0")
		CHANGED_FILES=$((CHANGED_FILES + changed))

		# Process ESLint with better memory management and timeout
		if ! NODE_OPTIONS="${node_memory}" timeout 300 npx eslint --ext .js,.jsx,.ts,.tsx src/ --fix --max-warnings 0 2>&1 | tee /tmp/eslint_output; then
			warning "ESLint fixing failed or timed out in web/ directory"
			exit_code=1
			local eslint_errors
			eslint_errors=$(grep -c "error\|warning" /tmp/eslint_output 2>/dev/null || echo "0")
			ERROR_FILES=$((ERROR_FILES + eslint_errors))
		fi
	fi

	cd ..
	if [[ ${exit_code} -eq 0 ]]; then
		success "TypeScript/JavaScript formatting completed successfully"
	else
		warning "Some TypeScript/JavaScript formatting issues were found"
	fi

	return "${exit_code}"
}

# Function to format code
direct_format() {
	local check_only=${1:-0}
	local rustfmt_cmd=("rustfmt")
	local exit_code=0
	local processed=0
	local succeeded=0

	if [[ ${check_only} -eq 1 ]]; then
		rustfmt_cmd+=("--check")
		status "Checking code formatting..."
	else
		rustfmt_cmd+=("--edition" "2021")
		status "Formatting code..."
	fi

	# Find all Rust files, excluding known problematic files and directories
	local find_excludes=""
	for dir in "target" ".cargo" "build" "rust-ai-ide-multi-reality" "rust-ai-ide-lsp"; do
		find_excludes="${find_excludes} -path '*/${dir}' -prune -o"
	done

	# Build the complete find command as a string
	local find_cmd="find . \( ${find_excludes} -type f -name '*.rs' -not -name enterprise_security_validation.rs -not -name demo_integration.rs -not -name integration_build_script.rs \) -print0"

	# Count total files to process
	processed=$(eval "${find_cmd}" | grep -c $'\0' || echo "0")
	TOTAL_FILES=$((TOTAL_FILES + processed))

	if [[ ${processed} -eq 0 ]]; then
		success "No Rust files found to format"
		return 0
	fi

	# Run rustfmt in parallel using xargs
	local xargs_cmd=("xargs" "-0" "-P" "${PARALLEL_JOBS}" "-n1")
	xargs_cmd+=("${rustfmt_cmd[@]}")

	local output
	if [[ ${check_only} -eq 1 ]]; then
		output=$(eval "${find_cmd}" | eval "${xargs_cmd[@]}" 2>/dev/null)
	else
		output=$(eval "${find_cmd}" | eval "${xargs_cmd[@]}" 2>&1)
	fi
	local exit_code_cmd=$?
	echo "${output}" | tee /tmp/rustfmt_output
	if [[ ${exit_code_cmd} -eq 0 ]]; then
		if [[ ${check_only} -eq 0 ]]; then
			local changed
			changed=$(grep -c "Diff in" /tmp/rustfmt_output)
			CHANGED_FILES=$((CHANGED_FILES + changed))
		fi
		succeeded=${processed}
		success "Successfully processed ${succeeded} files using ${PARALLEL_JOBS} parallel jobs"
	else
		exit_code=1
		if [[ ${check_only} -eq 0 ]]; then
			ERROR_FILES=$((ERROR_FILES + processed))
		fi
		warning "Some files failed to format. Check the output for details."
	fi

	return "${exit_code}"
}

# Function to format code
format_code() {
	local check_only=${1:-0}

	# First, clean up whitespace
	clean_whitespace

	# Format Rust code
	direct_format "$@"

	# Format TypeScript/JavaScript if available
	if check_prettier_and_eslint; then
		format_js_ts "${check_only}"
	fi

	return $?
}

# Function to check for unstaged changes
check_unstaged_changes() {
	if ! git diff --quiet -- '*.rs'; then
		warning "Found unstaged changes in .rs files. Stashing them temporarily..."
		git stash push --keep-index --include-untracked -- '*.rs' || {
			error "Failed to stash changes. Please commit or stash them manually."
		}
		return 0
	fi
	return 1
}

# Function to restore stashed changes
restore_stashed_changes() {
	if [[ ${has_stashed} -eq 1 ]]; then
		status "Restoring stashed changes..."
		git stash pop --quiet || {
			warning "Failed to restore stashed changes. Check with 'git stash list' and 'git stash show -p'"
		}
	fi
}

# Function to show summary
show_summary() {
	echo -e "\n${GREEN}=== Formatting Summary ===${NC}"
	echo "Total files processed: ${TOTAL_FILES}"
	echo "Files changed: ${CHANGED_FILES}"
	if [[ ${ERROR_FILES} -gt 0 ]]; then
		echo "Files with formatting errors: ${ERROR_FILES}"
	fi
	echo "Processing time: ${PROCESSING_TIME} seconds"
	if [[ ${ERROR_FILES} -eq 0 && ${CHANGED_FILES} -gt 0 ]]; then
		echo "Status: Success - Changes applied"
	elif [[ ${ERROR_FILES} -eq 0 && ${CHANGED_FILES} -eq 0 ]]; then
		echo "Status: No changes needed"
	else
		echo "Status: Completed with errors"
	fi
	echo -e "\n${YELLOW}Note: Some complex files were excluded from formatting"
	echo "to avoid breaking functionality. These include:"
	echo "- rust-ai-ide-multi-reality/ (AR/VR integration)"
	echo "- rust-ai-ide-lsp/ (Language Server Protocol)"
	echo -e "- Various test and demo files${NC}\n"
}

# Main execution
main() {
	# Initialize variables
	local has_stashed=0
	START_TIME=$(date +%s)
	TOTAL_FILES=0
	CHANGED_FILES=0
	ERROR_FILES=0
	PROCESSING_TIME=0
	local needs_formatting=0

	# Check for required commands
	for cmd in git rustup cargo; do
		if command_exists "${cmd}"; then
			continue
		fi
		error "Required command '${cmd}' not found. Please install it first."
	done

	# Install rustfmt if needed
	install_rustfmt

	# Ensure correct toolchain is used
	check_toolchain

	# Check for unstaged changes
	if check_unstaged_changes; then
		has_stashed=1
	fi

	# First, check formatting without making changes
	if ! format_code 1; then
		needs_formatting=1
		warning "Code formatting issues found. Fixing them..."

		# Now actually format the code
		if ! format_code 0; then
			error "Failed to format code. Please check the output above for errors."
			END_TIME=$(date +%s)
			PROCESSING_TIME=$((END_TIME - START_TIME))
			show_summary
		fi

		# Check if formatting made any changes
		if ! git diff --quiet -- '*.rs'; then
			warning "Code formatting changes were made. Please review and commit them."
			git --no-pager diff --stat
			needs_formatting=1
		fi
	fi

	# Restore stashed changes if any
	if [[ ${has_stashed} -eq 1 ]]; then
		restore_stashed_changes
	fi

	# Show summary before exiting
	show_summary

	# Final status
	if [[ ${needs_formatting} -eq 0 ]]; then
		success "Code is properly formatted!"
		exit 0
	else
		warning "Code formatting issues were fixed. Please review and commit the changes."
		exit 1
	fi
}

# Run the main function
main "$@"