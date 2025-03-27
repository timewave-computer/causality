# Causality CLI Reference

This document provides a comprehensive reference for the Causality Command Line Interface (CLI).

## Installation

### Using Cargo

```bash
cargo install causality-cli
```

### Using Nix

```bash
nix-env -i causality-cli
```

### From Source

```bash
git clone https://github.com/timewave-team/causality.git
cd causality
cargo build --release --bin causality-cli
```

## Configuration

The CLI can be configured using:

1. Command-line flags
2. Environment variables
3. Configuration file

### Configuration File

By default, the CLI looks for a configuration file at `~/.causality/config.toml`. You can specify a different location using the `--config` flag.

Example configuration file:

```toml
[global]
endpoint = "http://localhost:8080"
trace_level = "info"
capability_file = "~/.causality/capability.json"

[profiles]
default = "local"

[profiles.local]
endpoint = "http://localhost:8080"
capability_file = "~/.causality/local_capability.json"

[profiles.testnet]
endpoint = "https://testnet.causality.io"
capability_file = "~/.causality/testnet_capability.json"
```

### Environment Variables

You can override configuration using environment variables:

- `CAUSALITY_ENDPOINT`: API endpoint URL
- `CAUSALITY_CAPABILITY`: Path to capability file
- `CAUSALITY_PROFILE`: Profile to use from config
- `CAUSALITY_TRACE_LEVEL`: Logging level (error, warn, info, debug, trace)

## Global Options

The following options apply to all commands:

```
--config, -c <FILE>       Path to configuration file
--endpoint, -e <URL>      API endpoint URL
--capability <FILE>       Path to capability file
--profile <NAME>          Configuration profile to use
--trace-level <LEVEL>     Set the trace level (error, warn, info, debug, trace)
--output, -o <FORMAT>     Output format (json, yaml, table)
--help, -h                Print help information
--version, -V             Print version information
```

## Commands

### Resource Commands

#### resource list

List resources with optional filtering.

```bash
causality resource list [OPTIONS]
```

Options:
```
--state <STATE>           Filter by resource state
--resource-logic <HASH>   Filter by resource logic
--limit <LIMIT>           Maximum number of resources to return
--offset <OFFSET>         Number of resources to skip
```

Example:
```bash
causality resource list --state active --limit 10
```

#### resource get

Get a resource by ID.

```bash
causality resource get <RESOURCE_ID>
```

Example:
```bash
causality resource get 0x123456789abcdef
```

#### resource create

Create a new resource.

```bash
causality resource create [OPTIONS]
```

Options:
```
--logic <HASH>            Resource logic hash
--domain <DOMAIN>         Fungibility domain
--quantity <QUANTITY>     Resource quantity
--metadata <FILE>         JSON file with resource metadata
```

Example:
```bash
causality resource create --logic 0xabcdef --domain 1 --quantity 100 --metadata metadata.json
```

#### resource update

Update an existing resource.

```bash
causality resource update <RESOURCE_ID> [OPTIONS]
```

Options:
```
--state <STATE>           New resource state
--metadata <FILE>         JSON file with updated metadata
```

Example:
```bash
causality resource update 0x123456789abcdef --state locked --metadata updated_metadata.json
```

### Effect Commands

#### effect execute

Execute an effect.

```bash
causality effect execute [OPTIONS]
```

Options:
```
--type <TYPE>             Effect type
--params <FILE>           JSON file with effect parameters
--wait                    Wait for effect completion
--timeout <SECONDS>       Timeout in seconds when waiting
```

Example:
```bash
causality effect execute --type transfer --params transfer_params.json --wait
```

#### effect status

Check the status of an effect.

```bash
causality effect status <EFFECT_ID>
```

Example:
```bash
causality effect status 0x123456789abcdef
```

#### effect list

List effects with optional filtering.

```bash
causality effect list [OPTIONS]
```

Options:
```
--resource <RESOURCE_ID>  Filter by related resource ID
--type <TYPE>             Filter by effect type
--status <STATUS>         Filter by effect status
--limit <LIMIT>           Maximum number of effects to return
--offset <OFFSET>         Number of effects to skip
```

Example:
```bash
causality effect list --resource 0x123456789abcdef --status completed
```

### Program Commands

#### program deploy

Deploy a new program.

```bash
causality program deploy [OPTIONS]
```

Options:
```
--file <FILE>             Program definition file
--name <NAME>             Program name
--initial-state <FILE>    JSON file with initial state
```

Example:
```bash
causality program deploy --file my_program.json --name "My Program" --initial-state initial.json
```

#### program get

Get program information.

```bash
causality program get <PROGRAM_ID>
```

Example:
```bash
causality program get 0x123456789abcdef
```

#### program state

Get program state.

```bash
causality program state <PROGRAM_ID>
```

Example:
```bash
causality program state 0x123456789abcdef
```

### Fact Commands

#### fact get

Get a fact by ID.

```bash
causality fact get <FACT_ID>
```

Example:
```bash
causality fact get 0x123456789abcdef
```

#### fact query

Query facts based on criteria.

```bash
causality fact query [OPTIONS]
```

Options:
```
--type <TYPE>             Fact type
--domain <DOMAIN>         Domain ID
--params <FILE>           JSON file with query parameters
```

Example:
```bash
causality fact query --type balance --domain ethereum:mainnet --params query_params.json
```

### Domain Commands

#### domain list

List available domains.

```bash
causality domain list
```

Example:
```bash
causality domain list
```

#### domain get

Get information about a specific domain.

```bash
causality domain get <DOMAIN_ID>
```

Example:
```bash
causality domain get ethereum:mainnet
```

### Capability Commands

#### capability create

Create a new capability.

```bash
causality capability create [OPTIONS]
```

Options:
```
--resource <RESOURCE_ID>  Target resource ID
--actions <ACTIONS>       Allowed actions (comma-separated)
--expires <TIMESTAMP>     Expiration timestamp
--output-file <FILE>      Output file for the capability
```

Example:
```bash
causality capability create --resource 0x123456789abcdef --actions read,update --expires 2023-12-31T23:59:59Z
```

#### capability verify

Verify a capability.

```bash
causality capability verify <CAPABILITY_ID>
```

Example:
```bash
causality capability verify 0x123456789abcdef
```

### Debug Commands

#### debug trace

Trace the execution of an effect.

```bash
causality debug trace <EFFECT_ID>
```

Example:
```bash
causality debug trace 0x123456789abcdef
```

#### debug replay

Replay an effect execution.

```bash
causality debug replay <EFFECT_ID>
```

Example:
```bash
causality debug replay 0x123456789abcdef
```

## Error Handling

The CLI uses exit codes to indicate error conditions:

- 0: Success
- 1: General error
- 2: Configuration error
- 3: Network error
- 4: Authentication error
- 5: Resource error
- 6: Effect error
- 7: Timeout error

Error messages are printed to stderr and are formatted according to the selected output format.

## Environment Setup

### Development Environment

For development use, you can set up a local environment:

```bash
causality dev setup
```

This command:
1. Creates a local development configuration
2. Generates a development capability token
3. Sets up a local database
4. Starts a local server

### Mock Mode

For testing, you can use mock mode which doesn't require a server:

```bash
causality --mock effect execute --type transfer --params transfer_params.json
```

## Shell Completion

Generate shell completion scripts:

```bash
# Bash
causality completion bash > ~/.bash_completion.d/causality

# Zsh
causality completion zsh > ~/.zfunc/_causality

# Fish
causality completion fish > ~/.config/fish/completions/causality.fish
```

## Examples

### Creating and Transferring Resources

```bash
# Create two resources
RESOURCE_A=$(causality resource create --logic 0xabcdef --domain 1 --quantity 100 --metadata metadata.json --output json | jq -r .id)
RESOURCE_B=$(causality resource create --logic 0xabcdef --domain 1 --quantity 0 --metadata metadata.json --output json | jq -r .id)

# Create transfer effect parameters
cat > transfer_params.json << EOF
{
  "fromResourceId": "$RESOURCE_A",
  "toResourceId": "$RESOURCE_B",
  "amount": 50
}
EOF

# Execute transfer effect
causality effect execute --type transfer --params transfer_params.json --wait
```

### Querying Blockchain State

```bash
# Create query parameters
cat > balance_query.json << EOF
{
  "account": "0x123456789abcdef",
  "asset": "ETH"
}
EOF

# Query balance fact
causality fact query --type balance --domain ethereum:mainnet --params balance_query.json
```

## Further Resources

- [CLI Repository](https://github.com/timewave-team/causality-cli)
- [API Documentation](rest.md)
- [Installation Guide](../../guides/getting-started.md)
- [Advanced Usage Examples](../../guides/advanced-cli-usage.md) 