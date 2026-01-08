# CLAUDE.md

This file provides guidance for Claude Code when working with the Golt ECS Framework repository.

## Project Overview

Golt is a lightweight Entity Component System (ECS) framework for building Solana programs. It provides CLI tooling, procedural macros, and runtime utilities to eliminate boilerplate when developing ECS-based smart contracts on Solana using the Pinocchio framework.

## Repository Structure

```
golt/
├── ecs-cli/          # CLI tool (golt binary) for project scaffolding and management
├── ecs-macros/       # Procedural macros (#[derive(Component)], #[derive(System)])
├── ecs-runtime/      # Runtime library with traits, error handling, and account utilities
├── ecs-registry/     # Entity Registry program for centralized entity management
└── examples/         # Example projects (currently empty)
```

## Build Commands

```bash
# Build all workspace crates
cargo build

# Build with release optimizations
cargo build --release

# Run the CLI tool
cargo run -p golt -- <command>

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy
```

## CLI Commands (golt)

```bash
golt init <name>                          # Initialize new Golt project
golt new component <name> [--seed <seed>] # Create new component
golt new system <name>                    # Create new system
golt generate ts [--output <dir>]         # Generate TypeScript bindings
golt generate keypair <name>              # Generate program keypair
golt build [--sbf true]                   # Build all programs (SBF target by default)
golt test [name]                          # Run tests (optionally for specific program)
golt deploy <name> [--url] [--keypair]    # Deploy program to Solana
golt list                                 # List components and systems
```

## Key Technologies

- **Pinocchio**: Lightweight Solana framework (primary runtime dependency)
- **syn/quote/proc-macro2**: Procedural macro infrastructure
- **clap**: CLI argument parsing
- **toml/serde**: Configuration file handling

## Architecture

### Three-Crate Design

1. **ecs-macros**: Derive macros that generate instruction enums, pack/unpack serialization, and PDA derivation code
2. **ecs-runtime**: Core traits (`Component`, `InstructionData`), account wrappers, error types, and validation macros
3. **golt (ecs-cli)**: Project scaffolding, code generation, and build orchestration

### Core Concepts

- **Components**: Type-safe Solana account data with discriminator-based PDAs
- **Systems**: Programs that query and modify components
- **PDAs**: Deterministic addresses derived from `[SEED, entity_pubkey, bump]`

## Code Patterns

### Component Definition
Components use `#[derive(Component)]` and implement automatic serialization:
- 8-byte discriminator (from seed string)
- Little-endian field encoding
- 1-byte bump at end

### Instruction Encoding
- First byte: instruction tag (0-255)
- Remaining bytes: little-endian parameters
- Supports primitives, Pubkey, and fixed-size arrays

### Error Handling
Use the `GoltError` enum and validation macros:
- `require!(condition, error)` - Assert condition
- `require_signer!(account)` - Verify signer
- `require_owned_by!(account, program_id)` - Verify ownership

## Key Files

- `ecs-cli/src/main.rs` - CLI entry point
- `ecs-cli/src/config.rs` - golt.toml configuration
- `ecs-cli/src/templates/mod.rs` - Code generation templates
- `ecs-runtime/src/component.rs` - Component trait
- `ecs-runtime/src/error.rs` - Error types and macros
- `ecs-macros/src/component.rs` - Component derive macro
- `ecs-macros/src/instruction.rs` - Instruction generation

## Configuration

Projects use `golt.toml` for configuration:
```toml
[project]
name = "my-project"
version = "0.1.0"
components_dir = "programs/components"
systems_dir = "programs/systems"
keypairs_dir = "keypairs"

[[components]]
name = "health"
seed = "health"
program_id = "..."
```

## Development Notes

- This is v0.1.0 - the framework is new and evolving
- TypeScript generation parses Rust source files to generate accurate bindings
- Instruction encoding doesn't support nested structs
- Testing relies on mollusk-svm for integration tests
- Full MagicBlock Ephemeral Rollups delegation support via `Delegatable` trait

## Known Issues

| Issue | Workaround |
|-------|------------|
| Pinocchio CPI requires matching account arrays | Don't include program in account_infos |
