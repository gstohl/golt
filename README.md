# Golt

A lightweight Entity Component System (ECS) framework for building Solana programs.

Golt provides CLI tooling, procedural macros, and runtime utilities to eliminate boilerplate when developing ECS-based smart contracts on Solana using the [Pinocchio](https://github.com/anza-xyz/pinocchio) framework.

## Features

- **Zero-dependency runtime** - Built on Pinocchio for minimal binary size
- **Code generation** - Scaffold components and systems with a single command
- **Type-safe PDAs** - Automatic PDA derivation and verification
- **Pack/Unpack serialization** - No serde overhead, raw byte manipulation
- **TypeScript bindings** - Auto-generated client code from Rust sources
- **MagicBlock Ephemeral Rollups** - First-class delegation support for gasless gameplay

## Installation

```bash
cargo install --git https://github.com/gstohl/golt ecs-cli
```

Or build from source:

```bash
git clone https://github.com/gstohl/golt
cd golt
cargo install --path ecs-cli
```

## Quick Start

### 1. Create a new project

```bash
golt init my-game
cd my-game
```

This creates:
```
my-game/
├── golt.toml              # Project configuration
├── Cargo.toml             # Workspace configuration
├── programs/
│   ├── core/              # Shared seeds and discriminators
│   ├── components/        # Component programs
│   └── systems/           # System programs
└── keypairs/              # Program keypairs (gitignored)
```

### 2. Create a component

```bash
golt new component health
```

This generates a complete component program with:
- State definition (`state.rs`)
- Instructions (`instruction.rs`)
- Processor (`processor.rs`)
- Entrypoint (`entrypoint.rs`)
- Error types (`error.rs`)

### 3. Create a system

```bash
golt new system combat
```

Systems are programs that operate on multiple components.

### 4. Build

```bash
golt build
```

Builds all programs for Solana (SBF target).

### 5. Deploy

```bash
golt generate keypair health
golt deploy health --url localhost
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `golt init <name>` | Initialize a new Golt project |
| `golt new component <name>` | Create a new component |
| `golt new system <name>` | Create a new system |
| `golt build` | Build all programs (SBF) |
| `golt build --sbf false` | Build without SBF (regular Rust) |
| `golt test` | Run tests for all programs |
| `golt test <name>` | Run tests for specific program |
| `golt deploy <name>` | Deploy a program to Solana |
| `golt deploy <name> --url <rpc>` | Deploy to specific RPC endpoint |
| `golt generate keypair <name>` | Generate program keypair |
| `golt generate ts` | Generate TypeScript bindings |
| `golt list` | List all components and systems |

## Architecture

### ECS Concepts

- **Entity**: A unique identifier (any Solana keypair)
- **Component**: Data attached to an entity (stored in PDAs)
- **System**: Logic that operates on components

### Component Structure

Components are Solana programs that store data in PDAs derived from:
```
PDA = ["seed", entity_pubkey, bump]
```

Example component:
```rust
#[repr(C)]
pub struct Health {
    pub discriminator: [u8; 8],  // "health\0\0"
    pub entity: [u8; 32],        // Entity pubkey
    pub current: u64,            // Current HP
    pub max: u64,                // Max HP
    pub bump: u8,                // PDA bump
}
```

### System Structure

Systems are programs that read/write multiple components:
```rust
// Combat system reads attacker health, writes target health
fn deal_damage(attacker: &Health, target: &mut Health, amount: u64) {
    if attacker.current > 0 {
        target.current = target.current.saturating_sub(amount);
    }
}
```

## Configuration

Projects are configured via `golt.toml`:

```toml
[project]
name = "my-game"
version = "0.1.0"
components_dir = "programs/components"
systems_dir = "programs/systems"
keypairs_dir = "keypairs"

[[components]]
name = "health"
seed = "health"
program_id = "He4LthXXX..."  # Set after deploy

[[components]]
name = "position"
seed = "position"

[[systems]]
name = "combat"
program_id = "ComBatXXX..."
```

## Runtime Library

The `golt-runtime` crate provides:

### Component Trait

```rust
pub trait Component: Sized {
    const DISCRIMINATOR: [u8; 8];
    const SEED: &'static [u8];
    const SIZE: usize;

    fn unpack(data: &[u8]) -> Option<Self>;
    fn pack(&self, data: &mut [u8]);
    fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8);
}
```

### Delegatable Trait (Ephemeral Rollups)

```rust
pub trait Delegatable: Component {
    fn get_entity(&self) -> &Pubkey;
    fn get_bump(&self) -> u8;
    fn delegation_seeds(&self) -> Vec<&[u8]>;
}
```

### Error Handling

```rust
use golt_runtime::{require, require_signer, require_writable, GoltError};

fn process(accounts: &[AccountInfo]) -> ProgramResult {
    let payer = &accounts[0];
    require_signer!(payer);
    require_writable!(payer, GoltError::AccountNotWritable);
    require!(amount > 0, GoltError::InvalidAccountData);
    Ok(())
}
```

### Entity Management

```rust
use golt_runtime::{create_entity, load_entity, deactivate_entity};

// Create entity with ID
let entity = create_entity(payer, entity_account, owner, entity_id, program_id)?;

// Load existing entity
let entity = load_entity(entity_account)?;

// Deactivate entity
deactivate_entity(entity_account)?;
```

### Delegation (MagicBlock Ephemeral Rollups)

```rust
use golt_runtime::delegation::{delegate_account, undelegate, is_delegated, DelegateConfig};

// Check if delegated
if is_delegated(account) {
    // Running on ER
}

// Delegate to ephemeral rollup
let config = DelegateConfig {
    commit_frequency_ms: 10000,
    validator: Some(validator_pubkey),
};
delegate_account(accounts, seeds, bump, config)?;

// Undelegate back to L1
undelegate(accounts)?;
```

## TypeScript Bindings

Generate TypeScript bindings:

```bash
golt generate ts --output generated
```

This creates:
```typescript
// generated/health.ts
export const HEALTH_PROGRAM_ID = new PublicKey('...');
export const HEALTH_SEED = 'health';
export const HEALTH_SIZE = 57;

export interface Health {
  entity: PublicKey;
  current: bigint;
  max: bigint;
  bump: number;
}

export function deriveHealthPDA(entity: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(HEALTH_SEED), entity.toBuffer()],
    HEALTH_PROGRAM_ID
  );
}

export function parseHealth(data: Buffer): Health { ... }
export async function fetchHealth(connection: Connection, address: PublicKey): Promise<Health | null> { ... }
export function createHealthInitInstruction(...): TransactionInstruction { ... }
```

## Project Structure

```
golt/
├── ecs-cli/           # CLI tool
│   └── src/
│       ├── commands/  # CLI commands (init, build, deploy, etc.)
│       ├── config.rs  # golt.toml parsing
│       ├── parser.rs  # Rust source parsing for TS gen
│       └── templates/ # Code generation templates
├── ecs-macros/        # Procedural macros
│   └── src/
│       ├── component.rs  # #[derive(Component)]
│       ├── system.rs     # #[derive(System)]
│       └── instruction.rs # Instruction generation
├── ecs-runtime/       # Runtime library
│   └── src/
│       ├── component.rs  # Component trait
│       ├── delegation.rs # ER delegation helpers
│       ├── entity.rs     # Entity helpers
│       ├── account.rs    # Account utilities
│       ├── error.rs      # Error types & macros
│       └── pda.rs        # PDA derivation
├── ecs-registry/      # Entity Registry program (optional)
│   └── src/
│       ├── state.rs      # Entity state (id, owner, active)
│       ├── instruction.rs # Create, Transfer, Deactivate
│       ├── processor.rs  # Instruction handlers
│       └── error.rs      # Registry errors
└── examples/          # Example projects
```

## Entity Registry (Optional)

The Entity Registry is an optional Solana program that provides centralized entity management. It must be deployed before your game programs if you want to use it.

### Deploying the Registry

```bash
# Build the registry
cargo build-sbf -p golt-registry

# Generate keypair
solana-keygen new -o registry-keypair.json

# Deploy
solana program deploy target/deploy/golt_registry.so --program-id registry-keypair.json --url localhost
```

### Registry Instructions

| Instruction | Accounts | Description |
|-------------|----------|-------------|
| `Create(entity_id: u64)` | payer, entity_pda, system_program | Create entity, payer becomes owner |
| `Transfer` | owner, entity_pda, new_owner | Transfer ownership |
| `Deactivate` | owner, entity_pda | Mark entity inactive |

### Entity PDA

Entities are stored in PDAs derived as:
```
PDA = ["entity", entity_id (u64 le bytes)]
```

### When to Use

- **With Registry**: Centralized entity ownership, transferable entities, entity deactivation
- **Without Registry**: Entities are just keypairs, components attach directly via their own PDAs

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| pinocchio | 0.8 | Solana runtime (zero-dep) |
| pinocchio-pubkey | 0.2 | Program ID macros |
| pinocchio-system | 0.2 | System program CPI |
| ephemeral-rollups-pinocchio | 0.7 | MagicBlock ER integration |

## Known Issues

| Issue | Workaround |
|-------|------------|
| Pinocchio CPI requires matching account arrays | Don't include program in account_infos |

## Contributing

Contributions are welcome! Please open an issue or PR.

## License

MIT
