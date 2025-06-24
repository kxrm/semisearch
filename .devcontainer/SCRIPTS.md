# DevContainer Scripts Guide

## Core Scripts

### `setup.sh`
- **Purpose**: Main setup script that runs when the container is created
- **When it runs**: Automatically via `postCreateCommand` in devcontainer.json
- **What it does**:
  - Installs essential system packages
  - Verifies Rust installation
  - Creates project structure
  - Sets up helpful aliases and functions
  - Creates initial Cargo.toml and Justfile if they don't exist

### `scripts/new-developer-setup.sh`
- **Purpose**: Interactive onboarding for new developers
- **When to run**: Manually when starting development
- **What it does**:
  - Offers different development approaches (MVP, Enhanced, Full)
  - Creates appropriate Cargo.toml based on chosen approach
  - Provides learning resources and next steps

## Maintenance Scripts

### `update.sh`
- **Purpose**: Update all development tools and dependencies
- **When to run**: Periodically to keep tools current
- **What it does**:
  - Updates system packages
  - Updates Rust toolchain
  - Updates installed Cargo tools
  - Updates Python ML libraries
  - Cleans up caches

### `install-more-tools.sh`
- **Purpose**: Install additional optional development tools
- **When to run**: After initial setup if you need more tools
- **What it offers**:
  - Rust development tools (cargo-edit, cargo-audit, etc.)
  - Modern CLI tools (ripgrep, bat, exa, fd)
  - Performance tools (hyperfine, tokei)
  - Cross-compilation tools
  - Python ML libraries

### `test-setup.sh`
- **Purpose**: Validate that the development environment is properly configured
- **When to run**: After setup or when troubleshooting
- **What it checks**:
  - Core tools (Rust, Git, SQLite, Python)
  - Rust development tools
  - Optional CLI tools
  - Project structure

## Root Directory Scripts

### `update-dev-env.sh`
- **Purpose**: Smart update script that works both inside and outside containers
- **When to run**: When you want to update your development environment
- **What it does**:
  - Detects if running in container or host
  - Calls appropriate update commands 