# StarForge Architecture Documentation

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Directory Structure](#directory-structure)
4. [Core Components](#core-components)
5. [Data Flow](#data-flow)
6. [Module Descriptions](#module-descriptions)
7. [Design Patterns](#design-patterns)
8. [Extension Points](#extension-points)

---

## Overview

StarForge is a command-line interface (CLI) tool built in Rust for Stellar and Soroban blockchain development. It provides a comprehensive suite of tools for wallet management, contract development, deployment, and testing.

### Key Design Principles

1. **Modularity**: Clear separation between commands, utilities, and plugins
2. **Type Safety**: Leveraging Rust's type system for reliability
3. **Extensibility**: Plugin system for third-party extensions
4. **User Experience**: Colored output, progress indicators, and helpful error messages
5. **Security**: Encrypted key storage, hardware wallet support, validation at every step

### Technology Stack

- **Language**: Rust 1.80+
- **CLI Framework**: clap 4.4.18 (derive API)
- **Cryptography**: ed25519-dalek, aes-gcm, argon2
- **Blockchain**: stellar-strkey, stellar-xdr
- **HTTP**: ureq 2.7.1
- **Serialization**: serde, toml, serde_json
- **UI**: colored, indicatif, dialoguer

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User CLI                             │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Main Entry Point                          │
│                     (src/main.rs)                            │
│  - Command parsing (clap)                                    │
│  - Banner display                                            │
│  - Command routing                                           │
│  - Telemetry tracking                                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        ▼                           ▼
┌──────────────────┐      ┌──────────────────┐
│    Commands      │      │   Utilities      │
│  (src/commands/) │◄────►│  (src/utils/)    │
└────────┬─────────┘      └──────────────────┘
         │                          │
         │                          ├─ Config Management
         │                          ├─ Cryptography
         │                          ├─ Horizon API
         │                          ├─ Soroban RPC
         │                          ├─ Template System
         │                          └─ Print Utilities
         │
         ├─ Wallet Management
         ├─ Contract Operations
         ├─ Network Management
         ├─ Transaction Handling
         ├─ Template Marketplace
         └─ Plugin System
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                   External Systems                           │
│  - Stellar Horizon API                                       │
│  - Soroban RPC                                               │
│  - Git Repositories                                          │
│  - Hardware Wallets (Ledger/Trezor)                         │
│  - File System (~/.starforge/)                               │
└─────────────────────────────────────────────────────────────┘
```

### Component Interaction

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Wallet  │────►│  Config  │◄────│ Network  │
│ Commands │     │  Utils   │     │ Commands │
└──────────┘     └────┬─────┘     └──────────┘
                      │
                      ▼
                ┌──────────┐
                │ Horizon  │
                │   API    │
                └──────────┘

┌──────────┐     ┌──────────┐     ┌──────────┐
│ Contract │────►│ Soroban  │◄────│  Deploy  │
│ Commands │     │   RPC    │     │ Commands │
└──────────┘     └──────────┘     └──────────┘

┌──────────┐     ┌──────────┐     ┌──────────┐
│ Template │────►│ Template │◄────│   New    │
│ Commands │     │  Utils   │     │ Commands │
└──────────┘     └──────────┘     └──────────┘
```

---

## Directory Structure

```
starforge/
├── src/
│   ├── main.rs                 # Entry point, CLI setup, command routing
│   ├── commands/               # Command implementations
│   │   ├── mod.rs             # Module exports
│   │   ├── wallet.rs          # Wallet management (create, list, show, fund)
│   │   ├── new.rs             # Project scaffolding
│   │   ├── contract.rs        # Contract operations (inspect, invoke)
│   │   ├── deploy.rs          # Contract deployment
│   │   ├── network.rs         # Network management
│   │   ├── tx.rs              # Transaction operations
│   │   ├── template.rs        # Template marketplace
│   │   ├── plugin.rs          # Plugin management
│   │   ├── monitor.rs         # Real-time monitoring
│   │   ├── shell.rs           # Interactive REPL
│   │   ├── test.rs            # Contract testing
│   │   ├── gas.rs             # Gas analysis
│   │   ├── benchmark.rs       # Performance benchmarking
│   │   ├── tutorial.rs        # Interactive tutorials
│   │   ├── completions.rs     # Shell completions
│   │   ├── invoke.rs          # Contract invocation
│   │   └── info.rs            # System information
│   ├── utils/                  # Utility modules
│   │   ├── mod.rs             # Module exports
│   │   ├── config.rs          # Configuration management
│   │   ├── crypto.rs          # Encryption/decryption
│   │   ├── horizon.rs         # Horizon API client
│   │   ├── soroban.rs         # Soroban RPC client
│   │   ├── templates.rs       # Template system
│   │   ├── print.rs           # Terminal output utilities
│   │   ├── hardware_wallet.rs # Hardware wallet integration
│   │   ├── multisig.rs        # Multi-signature support
│   │   ├── notifications.rs   # User notifications
│   │   ├── optimizer.rs       # WASM optimization
│   │   ├── profiler.rs        # Performance profiling
│   │   ├── repl.rs            # REPL implementation
│   │   ├── sandbox.rs         # Local contract execution
│   │   ├── stream.rs          # Event streaming
│   │   ├── telemetry.rs       # Usage analytics
│   │   ├── test_runner.rs     # Test execution
│   │   ├── tutorial_engine.rs # Tutorial system
│   │   └── mock_soroban.rs    # Mock Soroban for testing
│   └── plugins/                # Plugin system
│       ├── mod.rs             # Module exports
│       ├── interface.rs       # Plugin trait definitions
│       ├── loader.rs          # Dynamic library loading
│       └── registry.rs        # Plugin registry
├── templates/                  # Template marketplace
│   ├── registry.json          # Template metadata
│   ├── README.md              # Template documentation
│   └── examples/              # Example templates
│       └── simple-counter/    # Counter contract template
├── tutorials/                  # Interactive tutorials
│   └── hello-world/           # Hello world tutorial
├── benches/                    # Performance benchmarks
│   └── benchmarks.rs          # Criterion benchmarks
├── tests/                      # Integration tests
│   └── template_marketplace_test.rs
├── examples/                   # Usage examples
│   └── template_marketplace_usage.md
├── Cargo.toml                  # Rust package manifest
├── build.rs                    # Build script (completions)
├── Dockerfile                  # Container image
├── docker-compose.yml          # Docker composition
└── Documentation files
    ├── README.md              # Main documentation
    ├── Documentation.md       # Extended documentation
    ├── ARCHITECTURE.md        # This file
    ├── TEMPLATE_MARKETPLACE.md # Template feature docs
    ├── QUICK_START_TEMPLATES.md # Quick start guide
    └── IMPLEMENTATION_SUMMARY.md # Implementation details
```

---

## Core Components

### 1. Main Entry Point (`src/main.rs`)

**Purpose**: Application bootstrap and command routing

**Responsibilities**:
- Parse command-line arguments using clap
- Display ASCII banner (unless `--quiet`)
- Route commands to appropriate handlers
- Track telemetry for usage analytics
- Handle errors and exit codes

**Key Code Flow**:
```rust
fn main() {
    let cli = Cli::parse();           // Parse CLI args
    print_banner();                    // Show banner
    let result = match cli.command {   // Route to handler
        Commands::Wallet(cmd) => commands::wallet::handle(cmd),
        Commands::Template(cmd) => commands::template::handle(cmd),
        // ... other commands
    };
    track_telemetry();                 // Log usage
    handle_error(result);              // Exit with code
}
```

### 2. Command Layer (`src/commands/`)

**Purpose**: Implement user-facing commands

**Pattern**: Each command module follows this structure:
```rust
// Command definition with clap
#[derive(Subcommand)]
pub enum CommandName {
    SubCommand { /* args */ },
}

// Main handler
pub fn handle(cmd: CommandName) -> Result<()> {
    match cmd {
        SubCommand::Action { args } => action(args),
    }
}

// Individual action handlers
fn action(args: Args) -> Result<()> {
    // 1. Validate inputs
    // 2. Load configuration
    // 3. Call utility functions
    // 4. Display results
    // 5. Save state if needed
}
```

**Key Commands**:

#### Wallet Commands (`wallet.rs`)
- **Create**: Generate Ed25519 keypair, optionally encrypt, save to config
- **List**: Display all saved wallets with status
- **Show**: Display wallet details and live balance
- **Fund**: Request testnet funds via Friendbot
- **Sign**: Sign messages with local or hardware keys
- **Multisig**: Multi-signature account management

#### Template Commands (`template.rs`)
- **Search**: Find templates by keyword and tags
- **List**: Show all available templates
- **Show**: Display template details
- **Publish**: Add template to local registry
- **Remove**: Delete template from registry
- **Init**: Initialize with example templates

#### Network Commands (`network.rs`)
- **Show**: Display current network and available networks
- **Switch**: Change active network
- **Add**: Register custom network
- **Test**: Verify network connectivity

### 3. Utility Layer (`src/utils/`)

**Purpose**: Reusable business logic and external integrations

#### Configuration Management (`config.rs`)

**Data Structure**:
```rust
pub struct Config {
    pub version: String,
    pub network: String,
    pub wallets: Vec<WalletEntry>,
    pub networks: HashMap<String, NetworkConfig>,
    pub telemetry_enabled: Option<bool>,
}

pub struct WalletEntry {
    pub name: String,
    pub public_key: String,
    pub secret_key: Option<String>,  // Encrypted or plaintext
    pub network: String,
    pub created_at: String,
    pub funded: bool,
}
```

**Key Functions**:
- `load()` - Load config from `~/.starforge/config.toml`
- `save()` - Persist config to disk
- `validate_*()` - Input validation functions
- `migrate_config()` - Version migration system

#### Template System (`templates.rs`)

**Data Structure**:
```rust
pub struct TemplateRegistry {
    pub version: String,
    pub templates: Vec<TemplateEntry>,
}

pub struct TemplateEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub source: TemplateSource,
    pub downloads: u64,
    pub verified: bool,
}

pub enum TemplateSource {
    Git { url: String, branch: Option<String> },
    Local { path: String },
    Builtin { id: String },
}
```

**Key Functions**:
- `search_templates()` - Search with filtering
- `fetch_template()` - Download from source
- `validate_template_structure()` - Verify required files
- `publish_template()` - Add to registry

#### Cryptography (`crypto.rs`)

**Encryption Flow**:
```
User Password
     ↓
  Argon2 KDF (key derivation)
     ↓
  AES-256-GCM Key
     ↓
  Encrypt Secret Key
     ↓
  Store: "salt:nonce:ciphertext"
```

**Key Functions**:
- `prompt_password()` - Secure password input
- `encrypt_secret()` - AES-256-GCM encryption
- `decrypt_secret()` - Decrypt with password

#### Horizon API Client (`horizon.rs`)

**Endpoints**:
- `GET /accounts/{id}` - Fetch account details
- `GET /accounts/{id}/transactions` - Transaction history
- `POST /transactions` - Submit transaction
- `GET https://friendbot.stellar.org` - Fund testnet account

**Key Functions**:
- `fetch_account()` - Get account info
- `fetch_transactions_filtered()` - Get tx history with filters
- `fund_account()` - Request testnet funds
- `submit_payment_transaction()` - Submit signed tx

#### Soroban RPC Client (`soroban.rs`)

**RPC Methods**:
- `simulateTransaction` - Simulate contract call
- `sendTransaction` - Submit contract transaction
- `getLedgerEntries` - Fetch contract data
- `getEvents` - Stream contract events

**Key Functions**:
- `simulate_transaction()` - Test contract call
- `submit_transaction()` - Execute contract call
- `inspect_contract()` - Get contract details

### 4. Plugin System (`src/plugins/`)

**Architecture**:
```
Plugin Interface (Trait)
        ↓
Plugin Implementation (.so/.dylib/.dll)
        ↓
Plugin Loader (libloading)
        ↓
Plugin Registry (JSON)
        ↓
Command Execution
```

**Key Components**:
- `interface.rs` - Plugin trait definition
- `loader.rs` - Dynamic library loading
- `registry.rs` - Plugin metadata storage

---

## Data Flow

### 1. Wallet Creation Flow

```
User Command: starforge wallet create alice --encrypt
                    ↓
         Parse Arguments (clap)
                    ↓
         Validate Wallet Name
                    ↓
    Generate Ed25519 Keypair (rand + ed25519-dalek)
                    ↓
         Prompt for Password
                    ↓
    Derive Key (Argon2) → Encrypt Secret (AES-GCM)
                    ↓
         Load Config (config.rs)
                    ↓
    Add WalletEntry to Config
                    ↓
         Save Config (TOML)
                    ↓
         Display Success
```

### 2. Template Usage Flow

```
User Command: starforge new contract my-dex --template uniswap-v2 --from marketplace
                    ↓
         Parse Arguments
                    ↓
    Load Template Registry (templates.rs)
                    ↓
    Search for Template "uniswap-v2"
                    ↓
    Fetch Template (Git clone or local copy)
                    ↓
    Validate Structure (Cargo.toml, src/lib.rs)
                    ↓
    Copy to Destination
                    ↓
    Replace Placeholders:
      - {{PROJECT_NAME}} → my-dex
      - {{PROJECT_NAME_SNAKE}} → my_dex
      - {{PROJECT_NAME_PASCAL}} → MyDex
                    ↓
    Update Download Count
                    ↓
    Display Success + Next Steps
```

### 3. Contract Deployment Flow

```
User Command: starforge deploy --wasm contract.wasm --wallet deployer
                    ↓
         Validate WASM File
                    ↓
         Load Configuration
                    ↓
    Find Wallet "deployer"
                    ↓
    Fetch Account from Horizon
                    ↓
    Check XLM Balance
                    ↓
    Calculate WASM Hash
                    ↓
    Generate stellar CLI Command
                    ↓
    Display Command for User to Execute
```

### 4. Transaction Submission Flow

```
User Command: starforge tx send --from alice --to bob --amount 100
                    ↓
         Validate Inputs
                    ↓
    Load Wallet "alice"
                    ↓
    Fetch Source Account (Horizon)
                    ↓
    Check Balance
                    ↓
    Build Transaction XDR
                    ↓
    Decrypt Secret Key (if encrypted)
                    ↓
    Sign Transaction (ed25519)
                    ↓
    Submit to Horizon
                    ↓
    Display Transaction Hash
```

---

## Module Descriptions

### Commands Module

| File | Purpose | Key Functions |
|------|---------|---------------|
| `wallet.rs` | Wallet lifecycle management | `create()`, `list()`, `show()`, `fund()`, `sign()` |
| `template.rs` | Template marketplace | `search()`, `publish()`, `list()`, `show()` |
| `new.rs` | Project scaffolding | `scaffold_contract()`, `scaffold_dapp()` |
| `deploy.rs` | Contract deployment | `handle()`, `validate_wasm()` |
| `contract.rs` | Contract operations | `inspect()`, `invoke()` |
| `network.rs` | Network management | `show()`, `switch()`, `add()`, `test()` |
| `tx.rs` | Transaction handling | `send()`, `history()` |
| `monitor.rs` | Real-time monitoring | `monitor_contract()`, `monitor_wallet()` |
| `shell.rs` | Interactive REPL | `handle()` with REPL runner |
| `test.rs` | Contract testing | `run_contract_tests()` |
| `gas.rs` | Gas analysis | `analyze()`, `optimize()` |
| `plugin.rs` | Plugin management | `install()`, `list()`, `load()` |

### Utils Module

| File | Purpose | Key Functions |
|------|---------|---------------|
| `config.rs` | Config management | `load()`, `save()`, `validate_*()` |
| `templates.rs` | Template system | `search_templates()`, `fetch_template()` |
| `crypto.rs` | Encryption | `encrypt_secret()`, `decrypt_secret()` |
| `horizon.rs` | Horizon API | `fetch_account()`, `submit_transaction()` |
| `soroban.rs` | Soroban RPC | `simulate_transaction()`, `inspect_contract()` |
| `print.rs` | Terminal output | `success()`, `error()`, `kv()`, `separator()` |
| `hardware_wallet.rs` | HW wallet support | `connect()`, `sign()`, `get_address()` |
| `multisig.rs` | Multi-sig support | `create_account()`, `sign_transaction()` |
| `optimizer.rs` | WASM optimization | `analyze_wasm()`, `optimize_wasm()` |
| `profiler.rs` | Performance profiling | `Timer`, `Profiler` |
| `telemetry.rs` | Usage tracking | `track_event()`, `set_telemetry_enabled()` |

---

## Design Patterns

### 1. Command Pattern

Each command is a separate module with a `handle()` function:
```rust
pub fn handle(cmd: CommandEnum) -> Result<()> {
    match cmd {
        CommandEnum::Action1 { args } => action1(args),
        CommandEnum::Action2 { args } => action2(args),
    }
}
```

**Benefits**:
- Clear separation of concerns
- Easy to add new commands
- Testable in isolation

### 2. Repository Pattern

Configuration and data access abstracted through utility modules:
```rust
// config.rs acts as repository
pub fn load() -> Result<Config> { /* ... */ }
pub fn save(config: &Config) -> Result<()> { /* ... */ }
```

**Benefits**:
- Centralized data access
- Easy to change storage backend
- Consistent error handling

### 3. Strategy Pattern

Template sources use enum with different strategies:
```rust
pub enum TemplateSource {
    Git { url: String, branch: Option<String> },
    Local { path: String },
    Builtin { id: String },
}

pub fn fetch_template(source: &TemplateSource) -> Result<()> {
    match source {
        TemplateSource::Git { url, branch } => fetch_git(url, branch),
        TemplateSource::Local { path } => fetch_local(path),
        TemplateSource::Builtin { id } => fetch_builtin(id),
    }
}
```

### 4. Builder Pattern

Used in configuration and complex structures:
```rust
let stream = SorobanEventStream::new(rpc_url, contract_id)
    .with_poll_interval(5)
    .with_filter(event_types);
```

### 5. Facade Pattern

Print utilities provide simple interface to complex terminal operations:
```rust
p::header("Title");
p::kv("Key", "Value");
p::success("Done!");
```

---

## Extension Points

### 1. Adding New Commands

1. Create new file in `src/commands/`
2. Define command enum with clap attributes
3. Implement `handle()` function
4. Add to `src/commands/mod.rs`
5. Add to `Commands` enum in `src/main.rs`

### 2. Adding New Template Sources

1. Add variant to `TemplateSource` enum
2. Implement fetch logic in `fetch_template()`
3. Update validation if needed

### 3. Adding New Plugins

1. Implement `Plugin` trait
2. Compile as dynamic library
3. Register with `starforge plugin install`

### 4. Adding New Networks

```bash
starforge network add mynet \
  --horizon-url https://horizon.example.com \
  --soroban-rpc-url https://soroban.example.com
```

### 5. Extending Configuration

1. Add field to `Config` struct
2. Update `Default` implementation
3. Add migration in `migrate_config()`
4. Update version number

---

## Security Architecture

### 1. Key Storage

```
Plaintext Mode:
  Secret Key → Config File (NOT RECOMMENDED)

Encrypted Mode:
  Secret Key → Argon2 KDF → AES-256-GCM → Config File
```

### 2. Hardware Wallet Integration

```
User Request → HID API → Hardware Device → Signature
```

### 3. Input Validation

All user inputs validated before processing:
- Public keys: 56 chars, starts with 'G', base32
- Contract IDs: 56 chars, starts with 'C', base32
- Amounts: Positive numbers
- Wallet names: Alphanumeric + dash/underscore

### 4. Network Security

- HTTPS for all API calls
- Mainnet warnings for destructive operations
- Confirmation prompts for high-risk actions

---

## Performance Considerations

### 1. Configuration Caching

Config loaded once per command execution, cached in memory.

### 2. Template Registry

Registry loaded on-demand, not at startup.

### 3. Git Cloning

Shallow clones (`--depth 1`) for faster template fetching.

### 4. Parallel Operations

Independent operations can be parallelized (future enhancement).

---

## Error Handling Strategy

### 1. Result Type

All fallible operations return `Result<T, anyhow::Error>`:
```rust
pub fn operation() -> Result<()> {
    validate_input()?;
    perform_action()?;
    Ok(())
}
```

### 2. Context Addition

Errors enriched with context:
```rust
fs::read_to_string(&path)
    .with_context(|| format!("Failed to read {}", path.display()))?
```

### 3. User-Friendly Messages

Errors displayed with helpful suggestions:
```
✗ Error: Wallet 'alice' not found

Try: starforge wallet list
```

---

## Testing Strategy

### 1. Unit Tests

In-module tests for individual functions:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_public_key() {
        assert!(validate_public_key("GABC...").is_ok());
    }
}
```

### 2. Integration Tests

In `tests/` directory for end-to-end workflows.

### 3. Property-Based Testing

For validation functions (future enhancement).

---

## Future Architecture Enhancements

### 1. Async/Await

Convert blocking I/O to async for better performance:
```rust
pub async fn fetch_account(key: &str) -> Result<Account> {
    // Async HTTP request
}
```

### 2. Database Backend

Replace TOML config with SQLite for better querying:
```
Config File → SQLite Database
```

### 3. Remote Template Registry

Central server for global template sharing:
```
Local Registry → Remote API → Global Registry
```

### 4. WebAssembly Support

Run StarForge in browser via WASM.

### 5. GraphQL API

Expose StarForge functionality via GraphQL.

---

## Conclusion

StarForge's architecture is designed for:
- **Maintainability**: Clear module boundaries
- **Extensibility**: Plugin system and extension points
- **Reliability**: Type safety and error handling
- **Performance**: Efficient operations and caching
- **Security**: Encryption and validation throughout

The modular design allows for easy addition of new features while maintaining code quality and user experience.
