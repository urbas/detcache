# detcache [![builder](https://github.com/urbas/detcache/actions/workflows/build.yml/badge.svg)](https://github.com/urbas/detcache/actions/workflows/build.yml)

A minimalist utility for key-value caching. It assumes that the key is a
hexadecimal SHA-256 hash.

`detcache` can use different backends in which to store key-value data.
Currently supported backends:

- filesystem
- s3

`detcache` aims to be good at storing small amounts of data (in the order of 10s
or 100s of KB at most).

## Installation

Use nix (see [nix installation instructions](https://nixos.org/download/)):

```bash
nix run github:urbas/detcache# -- help
```

## Usage

### Basic Operations

- **Associate some data with a SHA-256 key**:

  ```bash
  echo -n <data> | detcache put <hexadecimal sha-256 hash>
  ```

- **Get the previously stored data**:

  ```bash
  detcache get <hexadecimal sha-256 hash>
  ```

### Example

Say you have deterministically built some small results based on some input
files. You can store the resulting data into a cache with `detcache` to avoid
rebuilding that data again.

```bash
# Calculate a hash of all inputs that might influence the resulting data
inputs_sha256=$(cat input.* | sha256sum | cut -f1 -d' ')

# Cache the data that was calculated from the inputs
echo -n expensive-result | detcache put $inputs_sha256

# Later, retrieve the data without recalculation
detcache get $inputs_sha256
# Output: expensive-result
```

Storing deterministically created results from known inputs is where this tool
got its name: `Deterministic Cache`.

### Configuration

`detcache` automatically looks for a configuration file at
`$XDG_CONFIG_HOME/detcache/config.toml` (or `~/.config/detcache/config.toml` if
XDG_CONFIG_HOME is not set). You can also specify a custom configuration file:

```bash
detcache --config path/to/config.toml get <inputs sha256 digest>
```

By default, `detcache` stores cache data in `$XDG_CACHE_HOME/detcache/` (or
`~/.cache/detcache/` if XDG_CACHE_HOME is not set). You can override this
behaviour by setting the `[caches.default]` setting in the configuration file.

In the configuration file, you can define other cache backends (like S3). See
[example-config.toml](example-config.toml) for a reference configuration.

## License

[MIT License](LICENSE)
