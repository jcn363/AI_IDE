#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔍 Checking workspace consistency...${NC}"

# Check if in the correct directory
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Please run this script from the workspace root${NC}"
    exit 1
fi

# Check for cargo-edit
if ! command -v cargo-set-version &> /dev/null; then
    echo -e "${YELLOW}⚠️ cargo-edit not found. Installing...${NC}"
    cargo install cargo-edit
fi

# Check for cargo-hakari
if ! command -v cargo-hakari &> /dev/null; then
    echo -e "${YELLOW}⚠️ cargo-hakari not found. Installing...${NC}"
    cargo install cargo-hakari
fi

# Check for cargo-udeps
if ! command -v cargo-udeps &> /dev/null; then
    echo -e "${YELLOW}⚠️ cargo-udeps not found. Installing...${NC}"
    cargo install cargo-udeps --locked
fi

# Update workspace-hack dependencies
echo -e "\n${GREEN}🔄 Updating workspace-hack dependencies...${NC}"
cargo hakari generate

# Check for unused dependencies
echo -e "\n${GREEN}🔍 Checking for unused dependencies...${NC}"
cargo +nightly udeps --workspace || true

# Check for outdated dependencies
echo -e "\n${GREEN}🔄 Checking for outdated dependencies...${NC}"
cargo outdated --workspace --exit-code 1 || echo -e "${YELLOW}⚠️ Some dependencies are outdated. Consider updating them.${NC}"

# Check for duplicate dependencies
echo -e "\n${GREEN}🔍 Checking for duplicate dependencies...${NC}"
cargo tree -d

# Verify workspace members
echo -e "\n${GREEN}📋 Verifying workspace members...${NC}"
for member in $(cargo metadata --format-version=1 --no-deps | jq -r '.workspace_members[]' | cut -d ' ' -f1); do
    crate_dir=$(echo "$member" | cut -d ' ' -f1 | cut -d ':' -f1)
    if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
        echo -e "${RED}❌ Missing Cargo.toml for $crate_dir${NC}"
    else
        # Check if crate depends on workspace-hack
        if ! grep -q "workspace-hack" "$crate_dir/Cargo.toml"; then
            echo -e "${YELLOW}⚠️ $crate_dir does not depend on workspace-hack${NC}"
        fi
        
        # Check for version mismatches
        crate_version=$(grep -m 1 '^version = ' "$crate_dir/Cargo.toml" | cut -d '\"' -f2)
        if [[ "$crate_version" != "0.1.0" ]]; then
            echo -e "${YELLOW}⚠️ $crate_dir has version $crate_version, expected 0.1.0${NC}"
        fi
    fi
done

echo -e "\n${GREEN}✅ Workspace consistency check completed!${NC}"
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Review and fix any issues reported above"
echo "2. Run 'cargo check --workspace' to verify everything builds"
echo "3. Run 'cargo test --workspace' to run all tests"
echo -e "\n${GREEN}Happy coding! 🚀${NC}"
