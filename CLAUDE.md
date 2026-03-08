# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run
cargo run

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Check without building
cargo check

# Lint
cargo clippy
```

## Architecture

This is an async IoT protocol gateway server built with Tokio. It listens on multiple TCP ports simultaneously, with each port handling a different IoT protocol.

### Module Structure

- **`server/config.rs`** — `ServerConfig` and `ListenerConfig` structs; config can be loaded from `config.toml` (TOML format) or constructed programmatically via `ServerConfig::example()`.
- **`error.rs`** — Central `IotError` enum using `thiserror`; `IotResult<T>` alias used throughout.
- **`protocol/traits.rs`** — Two core traits all protocol implementations must fulfill:
  - `FrameDetector`: detects/validates complete frames in a byte stream (`check_frame`, `detector_frame`)
  - `ProtocolParser`: parses a validated frame into a structured representation
- **`protocol/gb26875/`** — Implementation of the GB/T 26875 Chinese fire alarm standard protocol. `Gb26875FrameDetector` implements `FrameDetector`. Frame validation must check: start bytes, length field, checksum, and end bytes.

### Configuration

`config.toml` defines the listeners:
```toml
[[listeners]]
port = 8080
protocol = "Gb26875"
bind_addr = "0.0.0.0"
```

Each listener binds a `TcpListener` on the specified address/port and dispatches incoming connections to the appropriate protocol handler based on the `protocol` string.

### Adding a New Protocol

1. Create `src/protocol/<name>/mod.rs` and implement `FrameDetector` and/or `ProtocolParser` from `protocol::traits`.
2. Export the module from `src/protocol/mod.rs`.
3. Wire the protocol name string to its handler in `main.rs` (currently pending implementation).

### Key Dependencies

- `tokio` (full features) — async runtime and TCP networking
- `tokio-util` — codec utilities for framing
- `bytes` — `BytesMut` used as the primary buffer type in trait interfaces
- `encoding_rs` — GBK/GB18030 text decoding (used for Chinese-encoded content in GB26875 frames)
- `thiserror` — error derive macros
- `serde` + `toml` — config deserialization
