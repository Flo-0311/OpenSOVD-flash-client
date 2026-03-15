#!/usr/bin/env bash
# ADR-0005: Mandatory Test Coverage — Local enforcement script
#
# Usage: ./scripts/check-tests.sh
#
# This script verifies that:
#   1. All crates compile without warnings
#   2. All tests pass
#   3. Every library crate source file with `pub` items has a `#[cfg(test)]` block
#
# Run before committing to ensure compliance with ADR-0005.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "========================================"
echo " ADR-0005: Mandatory Test Coverage Check"
echo "========================================"
echo

# Step 1: Build with warnings as errors
echo -e "${YELLOW}[1/3] Building workspace (warnings = errors)...${NC}"
if RUSTFLAGS="-D warnings" cargo build --workspace 2>&1; then
    echo -e "${GREEN}  ✓ Build passed${NC}"
else
    echo -e "${RED}  ✗ Build failed${NC}"
    exit 1
fi
echo

# Step 2: Run all tests
echo -e "${YELLOW}[2/3] Running all tests...${NC}"
if cargo test --workspace 2>&1; then
    echo -e "${GREEN}  ✓ All tests passed${NC}"
else
    echo -e "${RED}  ✗ Tests failed${NC}"
    exit 1
fi
echo

# Step 3: Check that every lib source file with pub items has #[cfg(test)]
echo -e "${YELLOW}[3/3] Checking test module coverage...${NC}"
MISSING=0
LIB_CRATES=(
    "crates/sovd-core/src"
    "crates/sovd-client/src"
    "crates/sovd-plugin/src"
    "crates/sovd-workflow/src"
    "crates/sovd-observe/src"
    "crates/sovd-cli/src"
)

for crate_src in "${LIB_CRATES[@]}"; do
    if [ ! -d "$crate_src" ]; then
        continue
    fi
    while IFS= read -r -d '' file; do
        # Skip lib.rs and mod.rs (re-export modules, typically no logic)
        basename=$(basename "$file")
        if [[ "$basename" == "lib.rs" || "$basename" == "mod.rs" || "$basename" == "main.rs" ]]; then
            continue
        fi
        # Check if file has pub items
        if grep -q 'pub ' "$file"; then
            # Check if file has test module
            if ! grep -q '#\[cfg(test)\]' "$file"; then
                echo -e "${RED}  ✗ Missing tests: $file${NC}"
                MISSING=$((MISSING + 1))
            fi
        fi
    done < <(find "$crate_src" -name '*.rs' -print0)
done

if [ "$MISSING" -gt 0 ]; then
    echo
    echo -e "${RED}  ✗ $MISSING source file(s) missing #[cfg(test)] module${NC}"
    echo -e "${RED}    See ADR-0005: docs/adr/0005-mandatory-test-coverage.md${NC}"
    exit 1
else
    echo -e "${GREEN}  ✓ All source files have test modules${NC}"
fi

echo
echo -e "${GREEN}========================================"
echo -e " ADR-0005 compliance: PASSED"
echo -e "========================================${NC}"
