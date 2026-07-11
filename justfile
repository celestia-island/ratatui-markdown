# ratatui-markdown Build System
#
# Usage:
#   just <recipe>        - Run specified recipe
#   just --list          - List all available recipes
#   just --summary       - Briefly list all recipe names

set shell := ["bash", "-c"]
set windows-shell := ["bash.exe", "-c"]
set unstable
set lists

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
[script('bash')]
fetch URL='':
    #!/usr/bin/env bash
    set -euo pipefail
    out=.just/celestia-devtools.just
    mkdir -p .just
    if [ -n "{{URL}}" ]; then
      echo "[fetch] {{URL}} -> $out"
      curl -fsSL "{{URL}}" -o "$out"
    elif command -v celestia-devtools >/dev/null 2>&1; then
      src=$(celestia-devtools include-path)
      echo "[fetch] local bundle ($src) -> $out"
      cp "$src" "$out"
    else
      echo "[fetch] github raw -> $out"
      curl -fsSL "https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just" -o "$out"
    fi
    echo "[fetch] wrote $out"

# Python command
py := "python3"


default:
    @just --list

# ============================================================================
# Build
# ============================================================================

# Build with all features (debug)
build:
    @echo "  →  Building..."
    @cargo build --all-features

# Build with all features (release)
build-release:
    @echo "  →  Building (Release)..."
    @cargo build --all-features --release

# ============================================================================
# Code quality
# ============================================================================

# Format code with rustfmt
fmt:
    @echo "  →  Formatting code..."
    @cargo fmt --all

# Run Clippy checks
clippy:
    @echo "  →  Running Clippy..."
    @cargo clippy --all-targets --all-features -- -D warnings

# Type-check all features
check:
    @echo "  →  Checking..."
    @cargo check --all-features

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test:
    @echo "  →  Running tests..."
    @cargo test --all-features

# Run tests with output
test-verbose:
    @cargo test --all-features -- --nocapture

# Run tests for each feature combination
test-all:
    @echo "  →  Testing no default features..."
    @cargo test --no-default-features
    @echo "  →  Testing markdown only..."
    @cargo test --no-default-features --features markdown
    @echo "  →  Testing scroll only..."
    @cargo test --no-default-features --features scroll
    @echo "  →  Testing tree..."
    @cargo test --no-default-features --features tree
    @echo "  →  Testing preview (all)..."
    @cargo test --all-features

# ============================================================================
# Maintenance
# ============================================================================

# Clean build artifacts
clean:
    @echo "  →  Cleaning..."
    @cargo clean

# Update dependencies
update:
    @echo "  →  Updating dependencies..."
    @cargo update

# ============================================================================
# Utilities
# ============================================================================

# Enforce use statement grouping rules
enforce-use:
    @{{py}} scripts/utils/enforce_use_group.py

# Run all CI checks locally
ci:
    @echo "  →  Running format check..."
    @cargo fmt --all -- --check
    @echo "  →  Running Clippy..."
    @cargo clippy --all-targets --all-features -- -D warnings
    @echo "  →  Checking --no-default-features..."
    @cargo check --no-default-features
    @echo "  →  Running tests..."
    @cargo test --all-features
    @echo "  ✓  All CI checks passed"

# ============================================================================
# Documentation
# ============================================================================

# Open API documentation in browser
doc:
    @cargo doc --all-features --open
