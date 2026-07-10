# ratatui-markdown Build System
#
# Usage:
#   just <recipe>        - Run specified recipe
#   just --list          - List all available recipes
#   just --summary       - Briefly list all recipe names

set shell := ["bash", "-c"]
# `set windows-shell` only governs linewise (non-shebang) recipes on Windows.
# Shebang recipes bypass it and force `just` to call `cygpath` to translate the
# interpreter path — which Git for Windows keeps off PATH, so they die with
# "could not find cygpath executable". To avoid that, every multi-line recipe
# below uses the `[script('bash')]` attribute instead of a `#!` shebang:
# `[script]` resolves the interpreter via PATH (PATHEXT-aware) and never calls
# cygpath. See casey/just#2828 and the just manual (Script Recipes).
set windows-shell := ["bash.exe", "-c"]
# `set lists` enables which() (used by the imported celestia-devtools.just);
# `set unstable` gates it.
set unstable
set lists

# Python command
py := "python3"

import "./celestia-devtools.just"

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
