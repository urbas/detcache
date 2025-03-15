# nr (Nix Resolve)

A high-performance key-value store for fast resolution of Nix store paths. `nr`
eliminates expensive recalculations by caching associations between data and
their corresponding Nix store paths.

## Installation

Use flakes:

```bash
nix run github:urbas/nr# -- help
```

## Usage

### Basic Operations

- **Associate data that's been deterministically calculated from some inputs**:

  ```bash
  echo -n <calculated data> | nr put <sha-256 hash of inputs>
  ```

- **Get the previously stored data based on given inputs**:

  ```bash
  nr get <sha-256 hash of inputs>
  ```

### Examples

```bash
# Calculate a hash of all inputs that might influence the store path
inputs_sha256=$(cat flake.* | sha256sum | cut -f1 -d' ')

# Cache the store path that was calculated from the inputs
echo -n /nix/store/a1b2c3-my-derivation | nr put $inputs_sha256

# Later, retrieve the path without recalculation
nr get $inputs_sha256
# Output: /nix/store/a1b2c3-my-derivation
```

### Configuration

`nr` can be configured to use secondary caches (like S3) by providing a
configuration file:

```bash
nr --config path/to/config.toml get <inputs sha256 digest>
```

See [example-config.toml](example-config.toml) for a reference configuration.

## How It Works

`nr` operates on a simple but powerful principle:

1. It stores associations between SHA-256 hashes and Nix store paths
2. It retrieves these associations when queried with the same data

The tool assumes that relationships between the SHA-256 hash and Nix store paths
are:

- **Deterministic**: The same hash always produces the same output
- **Unique**: Each hash maps to a distinct output
- **Immutable**: Once established, mappings don't change

This allows `nr` to implement aggressive caching strategies for significant
performance gains in interactive and latency-sensitive Nix workflows.

## License

[MIT License](LICENSE)
