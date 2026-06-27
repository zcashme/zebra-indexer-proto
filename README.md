# zebra-indexer-proto

Rust bindings for Zebra's indexer gRPC interface.

**The design goal of this crate is the absolute best "it just builds" experience for downstream users**, combined with a first-class, maintainable story for the protocol itself.

## Philosophy

- Normal `cargo add zebra-indexer-proto && cargo build` must **just work**.
  - No `protoc` installation required.
  - No heavy build-time code generation dependencies pulled in.
- The generated code is committed (see `proto/__generated__/`).
- The bindings are just a copy of what Zebra generates internally — no redesign, no
  extra abstraction layer. Client *and* server types are re-exported verbatim.
- Clear, low-friction maintenance process when the proto changes in Zebra.

This pattern is heavily inspired by the `lightwallet-protocol` crate and how Zebra itself manages its internal gRPC bindings.

## Quick Start

```toml
[dependencies]
zebra-indexer-proto = "2.4"
```

No `tonic` dependency needed in your crate — `ZebraClient` pins the transport for you.

### Client

```rust
use zebra_indexer_proto::{ZebraClient, Empty};

let mut client = ZebraClient::connect("http://127.0.0.1:8230").await?;

// Chain tip change stream
let mut stream = client.chain_tip_change(Empty {}).await?.into_inner();
while let Some(tip) = stream.message().await? {
    println!("tip: height={} hash={:x}", tip.height, tip.hash);
}
```

### Implementing a Server

This crate re-exports the server trait verbatim from the generated bindings — i.e.
the same `Indexer` trait Zebra implements internally. It's here mainly so downstream
test suites can spin up a stub `Indexer` (or a third-party Zebra-compatible node can
serve the same gRPC interface). You don't implement anything new; you just fill in
the 4 generated methods exactly as Zebra does:

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

### Serialization (serde)

Every generated message type derives `serde::Serialize` and `serde::Deserialize`, so you can
re-emit received payloads as JSON, cache them with `bincode`, snapshot-test them, etc. — no
hand-rolled encoders:

```rust
use zebra_indexer_proto::BlockHashAndHeight;

let tip = BlockHashAndHeight { hash: vec![1; 32], height: 123456 };
let json = serde_json::to_string(&tip)?;           // {"hash":[1,1,…],"height":123456}
let back: BlockHashAndHeight = serde_json::from_str(&json)?;
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
   cargo build --features regenerate
   ```

3. Commit the changes to both `proto/indexer.proto` and the two files under `__generated__/`.

## Features

| Feature     | Effect |
|-------------|--------|
| (default)   | Uses committed generated code. Pure "it just builds" experience. |
| `regenerate` | Regenerates `proto/__generated__/*` from `proto/indexer.proto`. For maintainers only. |

## License

MIT OR Apache-2.0 (same as Zebra).