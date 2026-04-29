# Template Marketplace Usage Examples

This document provides practical examples of using the StarForge Template Marketplace feature.

## Quick Start

### 1. Initialize the Marketplace

First, initialize the template registry with example templates:

```bash
starforge template init
```

Output:
```
  ███████╗████████╗ █████╗ ██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
  ...
  
  ◆ Initialize Template Registry
  ─────────────────────────────────────────────────────────────────────────
  
  Adding example templates to the marketplace...
  
  ✓ Added: uniswap-v2
  ✓ Added: lending-pool
  ✓ Added: governance
  ✓ Added: multisig-wallet
  
  ✓ Template registry initialized with 4 example templates
  
  Browse templates:
    starforge template list
    starforge template search defi
```

### 2. Browse Available Templates

List all templates:

```bash
starforge template list
```

Output:
```
  ◆ Template Marketplace — All Templates
  ─────────────────────────────────────────────────────────────────────────
  
  4 template(s) available:
  
  1. uniswap-v2 ✓
     Uniswap V2 style automated market maker (AMM) DEX implementation
     1.0.0 • Stellar Community • 0 downloads
     Tags: defi, dex, amm, swap
  
  2. lending-pool ✓
     Decentralized lending and borrowing protocol with collateralization
     1.0.0 • Stellar Community • 0 downloads
     Tags: defi, lending, borrowing
  
  3. governance ✓
     DAO governance contract with proposal creation and voting mechanisms
     1.0.0 • Stellar Community • 0 downloads
     Tags: dao, governance, voting
  
  4. multisig-wallet ✓
     Multi-signature wallet with threshold-based transaction approval
     1.0.0 • Stellar Community • 0 downloads
     Tags: wallet, multisig, security
```

### 3. Search for Specific Templates

Search by keyword:

```bash
starforge template search defi
```

Search with tag filtering:

```bash
starforge template search dex --tags amm
```

### 4. View Template Details

```bash
starforge template show uniswap-v2
```

Output:
```
  ◆ Template: uniswap-v2
  ─────────────────────────────────────────────────────────────────────────
  
  uniswap-v2 ✓
  
  Description    : Uniswap V2 style automated market maker (AMM) DEX implementation
  Version        : 1.0.0
  Author         : Stellar Community
  Downloads      : 0
  Tags           : defi, dex, amm, swap
  
  Created        : 2025-01-01T00:00:00Z
  Updated        : 2025-01-01T00:00:00Z
  
  Source         : Git Repository
  URL            : https://github.com/stellar/soroban-examples
  Branch         : main
  
  ─────────────────────────────────────────────────────────────────────────
  
  Use this template:
    starforge new contract my-project --template uniswap-v2 --from marketplace
```

## Using Templates

### Create a Project from a Marketplace Template

```bash
starforge new contract my-dex --template uniswap-v2 --from marketplace
```

Output:
```
  ◆ Scaffolding from Marketplace: uniswap-v2
  ─────────────────────────────────────────────────────────────────────────
  
  Template       : uniswap-v2
  Version        : 1.0.0
  Author         : Stellar Community
  Description    : Uniswap V2 style automated market maker (AMM) DEX implementation
  
  ─────────────────────────────────────────────────────────────────────────
  
  [1/3] Fetching template...
  [2/3] Validating template structure...
  [3/3] Copying template to project directory...
  
  ✓ Contract 'my-dex' scaffolded from marketplace!
  
  Next steps:
    cd my-dex
    stellar contract build
    starforge deploy --wasm target/wasm32-unknown-unknown/release/my_dex.wasm
```

### Search and Use in One Workflow

```bash
# First, search for what you need
starforge new contract --search lending

# Then use the template you found
starforge new contract my-lending --template lending-pool --from marketplace
```

## Publishing Templates

### Publish a Simple Template

```bash
starforge template publish ./my-template \
  --name my-counter \
  --description "A simple counter contract" \
  --author "John Doe" \
  --tags "example,counter" \
  --version "1.0.0"
```

Output:
```
  ◆ Publish Template
  
  [1/3] Validating template structure...
  [2/3] Copying template to local registry...
  [3/3] Updating registry...
  
  ✓ Template 'my-counter' published successfully!
  
  ─────────────────────────────────────────────────────────────────────────
  Name           : my-counter
  Version        : 1.0.0
  Author         : John Doe
  Tags           : example, counter
  ─────────────────────────────────────────────────────────────────────────
  
  Others can now use your template:
    starforge new contract my-project --template my-counter --from marketplace
```

### Interactive Publishing

If you don't provide all options, StarForge will prompt you:

```bash
starforge template publish ./my-template
```

Interactive prompts:
```
Template description: A simple counter contract
Author name: John Doe
```

## Advanced Usage

### Creating a Template with Placeholders

1. Create your contract:
```bash
starforge new contract template-base
cd template-base
```

2. Edit `Cargo.toml` to use placeholders:
```toml
[package]
name = "{{PROJECT_NAME}}"
version = "0.1.0"
edition = "2021"
```

3. Edit `src/lib.rs` to use placeholders:
```rust
#[contract]
pub struct {{PROJECT_NAME_PASCAL}};

#[contractimpl]
impl {{PROJECT_NAME_PASCAL}} {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}
```

4. Test your template:
```bash
cargo test
stellar contract build
```

5. Publish it:
```bash
cd ..
starforge template publish ./template-base \
  --name my-hello-world \
  --description "Custom hello world template" \
  --author "Your Name" \
  --tags "example,hello-world"
```

### Using Local Templates

You can also use templates from local directories without publishing:

```bash
# The template will be copied from the local path
starforge template publish ./my-local-template
```

### Managing Templates

Remove a template:
```bash
starforge template remove my-counter
```

## Real-World Examples

### Example 1: DeFi Developer

```bash
# Search for DeFi templates
starforge template search defi --tags dex

# View details of interesting template
starforge template show uniswap-v2

# Create your DEX project
starforge new contract my-stellar-dex --template uniswap-v2 --from marketplace

# Build and test
cd my-stellar-dex
stellar contract build
cargo test

# Deploy to testnet
starforge deploy \
  --wasm target/wasm32-unknown-unknown/release/my_stellar_dex.wasm \
  --network testnet
```

### Example 2: DAO Builder

```bash
# Find governance templates
starforge template search governance

# Use the governance template
starforge new contract my-dao --template governance --from marketplace

# Customize and deploy
cd my-dao
# Edit src/lib.rs to customize voting logic
stellar contract build
starforge deploy --wasm target/wasm32-unknown-unknown/release/my_dao.wasm
```

### Example 3: Template Creator

```bash
# Create a new innovative contract
starforge new contract flash-loan-template

# Develop your contract
cd flash-loan-template
# ... implement flash loan logic ...
cargo test
stellar contract build

# Add placeholders for reusability
# Edit Cargo.toml and src/lib.rs to add {{PROJECT_NAME}} etc.

# Publish for others to use
cd ..
starforge template publish ./flash-loan-template \
  --name flash-loan \
  --description "Flash loan implementation for Soroban" \
  --author "DeFi Innovator" \
  --tags "defi,flash-loan,lending" \
  --version "1.0.0"

# Share with the community
echo "New flash loan template available!"
echo "starforge new contract my-flash-loan --template flash-loan --from marketplace"
```

## Tips and Best Practices

### For Template Users

1. **Always review code** - Check the template source before using
2. **Prefer verified templates** - Look for the ✓ badge
3. **Check version** - Use the latest stable version
4. **Read documentation** - Review the template's README
5. **Test thoroughly** - Test in a safe environment first

### For Template Authors

1. **Use clear placeholders** - Make it obvious what needs to be replaced
2. **Include documentation** - Add a comprehensive README
3. **Add tests** - Include working test cases
4. **Follow conventions** - Use Soroban best practices
5. **Version properly** - Use semantic versioning
6. **Tag appropriately** - Use relevant, searchable tags
7. **Keep it simple** - Focus on one clear use case

## Troubleshooting

### Template Not Found

```bash
starforge template show my-template
# Error: Template 'my-template' not found in registry
```

Solution: Check available templates with `starforge template list`

### Invalid Template Structure

```bash
starforge template publish ./bad-template
# Error: Template must contain Cargo.toml
```

Solution: Ensure your template has the required files:
- `Cargo.toml`
- `src/` directory
- `src/lib.rs`

### Git Clone Failed

```bash
starforge new contract my-project --template some-template --from marketplace
# Error: Git clone failed: repository not found
```

Solution: Check the template's source URL with `starforge template show some-template`

## Next Steps

- Explore the [Template Marketplace Documentation](../TEMPLATE_MARKETPLACE.md)
- Check out [example templates](../templates/examples/)
- Join the community to share your templates
- Contribute to the official template registry
