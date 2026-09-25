# ratatui-markdown Build System
#
# Usage:
#   just <recipe>        - Run specified recipe
#   just --list          - List all available recipes
#   just --summary       - Briefly list all recipe names

set shell := ["bash", "-c"]
# Windows: PowerShell (the 5.1 floor ships with every Windows; pwsh 7 is
# NOT assumed). Linewise recipes must stay PS-5.1-safe: no `&&` chains,
# `cd X; cmd` instead of `cd X && cmd`. Bash-only recipes use
# [script('bash')] and need Git Bash (or WSL) when actually run.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command", "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $PSDefaultParameterValues['*:Encoding']='utf8';"]
set unstable
set lists

# Repo definitions override the shared template's (imported above).
set allow-duplicate-recipes
set allow-duplicate-variables

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/git-bash-interop.just"
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
fetch URL='':
    {{ if os_family() == "windows" { "python" } else { "python3" } }} -c "import os; os.makedirs('.just', exist_ok=True)"
    {{ if URL != "" { "curl -fsSL " + URL + " -o .just/celestia-devtools.just" } else if which("celestia-devtools") != "" { "celestia-devtools fetch-just" } else { "curl -fsSL https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just -o .just/celestia-devtools.just" } }}
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
    just fmt-toml
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
