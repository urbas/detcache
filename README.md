# detcache [![builder](https://github.com/urbas/detcache/actions/workflows/build.yml/badge.svg)](https://github.com/urbas/detcache/actions/workflows/build.yml)

A minimalist utility for caching small results of deterministic calculations.

`detcache` eliminates expensive recalculations by caching calculation results
based on a SHA-256 hash of the calculation inputs. The calculation must be
deterministic and the result must be small (in the order of 10s of kBs) for this
to work.

## Installation

Use nix (see [nix installation instructions](https://nixos.org/download/)):

```bash
nix run github:urbas/detcache# -- help
```

## Usage

### Basic Operations

- **Associate results deterministically calculated from some inputs**:

  ```bash
  echo -n <calculated results> | detcache put <sha-256 hash of inputs>
  ```

- **Get the previously stored results based on given inputs**:

  ```bash
  detcache get <sha-256 hash of inputs>
  ```

### Examples

```bash
# Calculate a hash of all inputs that might influence the result
inputs_sha256=$(cat input.* | sha256sum | cut -f1 -d' ')

# Cache the result that was calculated from the inputs
echo -n expensive-result | detcache put $inputs_sha256

# Later, retrieve the path without recalculation
detcache get $inputs_sha256
# Output: expensive-result
```

### Configuration

`detcache` can be configured to use secondary caches (like S3) by providing a
configuration file:

```bash
detcache --config path/to/config.toml get <inputs sha256 digest>
```

See [example-config.toml](example-config.toml) for a reference configuration.

## How It Works

`detcache` operates on a simple but powerful principle:

1. It stores associations between SHA-256 hashes of inputs and calculation
   results.
2. It retrieves these associations when queried with the same inputs.

The tool assumes that relationships between the inputs and results are:

- **Deterministic**: The same input always produces the same result
- **Unique**: Each input maps to a distinct result
- **Immutable**: Once established, mappings don't change

This allows `detcache` to implement aggressive caching strategies for
significant performance gains in interactive and latency-sensitive workflows.

## License

[MIT License](LICENSE)
