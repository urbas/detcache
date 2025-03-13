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

- **Get a store path from data**:

  ```bash
  nr get < input.json
  ```

  Returns the Nix store path associated with the data from standard input.

- **Associate data with a store path**:
  ```bash
  nr put /nix/store/hash-name < input.json
  ```
  Associates the provided Nix store path with the data from standard input.

### Examples

```bash
# Cache a derivation result
cat my-derivation.json | nr put /nix/store/a1b2c3-my-derivation

# Later, retrieve the path without recalculation
cat my-derivation.json | nr get
# Output: /nix/store/a1b2c3-my-derivation
```

### Configuration

`nr` can be configured to use secondary caches (like S3) by providing a
configuration file:

```bash
nr --config path/to/config.toml get < input.json
```

See [example-config.toml](example-config.toml) for a reference configuration.

## How It Works

`nr` operates on a simple but powerful principle:

1. It calculates SHA-256 hashes of input data blobs
2. It stores associations between these hashes and Nix store paths
3. It retrieves these associations when queried with the same data

The tool assumes that relationships between data blobs and Nix store paths are:

- **Deterministic**: The same input always produces the same output
- **Unique**: Each distinct input maps to a distinct output
- **Immutable**: Once established, mappings don't change

This allows `nr` to implement aggressive caching strategies for significant
performance gains in interactive and latency-sensitive Nix workflows.

## License

[MIT License](LICENSE)
