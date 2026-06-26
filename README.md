# zebra-indexer-proto

Rust bindings for Zebra's indexer gRPC interface.

**The design goal of this crate is the absolute best "it just builds" experience for downstream users**, combined with a first-class, maintainable story for the protocol itself.

## Philosophy

- Normal `cargo add zebra-indexer-proto && cargo build` must **just work**.
  - No `protoc` installation required.
  - No heavy build-time code generation dependencies pulled in.
- The generated code is committed (see `proto/__generated__/`).
- Both **client** and **server** sides are first-class.
- Clear, low-friction maintenance process when the proto changes in Zebra.

This pattern is heavily inspired by the `lightwallet-protocol` crate and how Zebra itself manages its internal gRPC bindings.

## Quick Start

```toml
[dependencies]
zebra-indexer-proto = "0.1"
tonic = "0.14"
```

### Client

```rust
use zebra_indexer_proto::{IndexerClient, Empty};

let mut client = IndexerClient::connect("http://127.0.0.1:8230").await?;

// Chain tip change stream
let mut stream = client.chain_tip_change(Empty {}).await?.into_inner();
while let Some(tip) = stream.message().await? {
    println!("tip: height={} hash={:x}", tip.height, tip.hash);
}
```

### Implementing a Server

```rust
use zebra_indexer_proto::indexer_server::{Indexer, IndexerServer};
use zebra_indexer_proto::{BlockAndHash, BlockHashAndHeight, /* ... */ };

#[tonic::async_trait]
impl Indexer for MyService {
    type ChainTipChangeStream = ...;
    // implement the 4 methods...
}

let server = IndexerServer::new(my_service);
```

### Reflection (optional but recommended)

```rust
use zebra_indexer_proto::FILE_DESCRIPTOR_SET;

let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build_v1()
    .unwrap();
```

## Protocol Source & Versioning

The `proto/indexer.proto` is the source of truth and is kept in sync with:

> [ZcashFoundation/zebra](https://github.com/ZcashFoundation/zebra/blob/main/zebra-rpc/proto/indexer.proto)

This crate is versioned independently of Zebra. It tracks the gRPC interface, not Zebra's internal crate versions.

## Methods

| RPC                        | Type      | Purpose |
|----------------------------|-----------|---------|
| `ChainTipChange`           | stream    | Push `(hash, height)` on every best chain tip change |
| `NonFinalizedStateChange`  | stream    | Stream full blocks in the non-finalized state (with resume support) |
| `MempoolChange`            | stream    | Notify on tx added / invalidated / mined |
| `GetBlock`                 | unary     | Fetch a block by hash (32 bytes) or height (4 bytes big-endian) |

## Maintenance (for contributors)

The committed generated files live in:

```
proto/
├── indexer.proto
└── __generated__/
    ├── zebra.indexer.rpc.rs
    └── indexer_descriptor.bin
```

### Updating the bindings

1. Update the proto definition (usually by copying from Zebra at a release/tag):
   ```bash
   # Example
   cp ../zebra/zebra-rpc/proto/indexer.proto proto/indexer.proto
   ```

2. Regenerate (requires `protoc`):
   ```bash
   make regenerate
   # or
   cargo build --features regenerate
   ```

3. Commit the changes to both `proto/indexer.proto` and the two files under `__generated__/`.

A `Makefile` with helper targets is provided.

## Features

| Feature     | Effect |
|-------------|--------|
| (default)   | Uses committed generated code. Pure "it just builds" experience. |
| `regenerate` | Regenerates `proto/__generated__/*` from `proto/indexer.proto`. For maintainers only. |

## License

MIT OR Apache-2.0 (same as Zebra).