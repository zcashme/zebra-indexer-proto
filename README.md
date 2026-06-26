# zebra-indexer-proto

Rust bindings for Zebra's indexer gRPC interface.

## Overview

This crate provides auto-generated Rust bindings for the [Zebra](https://github.com/ZcashFoundation/zebra) indexer gRPC protocol buffers. These `.proto` files define the gRPC interface between external clients and Zebra's indexer RPC service.

## Features

- Generated Rust type definitions for all indexer protocol messages
- gRPC client implementation for connecting to Zebra nodes
- gRPC server traits for implementing custom indexer services
- Automatic code generation from `.proto` files via `tonic-prost-build`
- Zero Zebra dependencies — just `prost`, `tonic`, and `tonic-prost`

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
zebra-indexer-proto = "0.1"
tonic = "0.14"
```

Then use the generated types in your code:

```rust
use zebra_indexer_proto::IndexerClient;
use zebra_indexer_proto::Empty;

// Connect to a Zebra node's indexer gRPC (port 8230 by default)
let mut client = IndexerClient::connect("http://127.0.0.1:8230").await?;

// Subscribe to chain tip changes (streaming)
let mut stream = client.chain_tip_change(Empty {}).await?.into_inner();
while let Some(tip) = stream.message().await? {
    println!("chain tip: height={}, hash={:x}", tip.height, tip.hash);
}
```

## Protocol Source

The `indexer.proto` is vendored from Zebra's `zebra-rpc/proto/indexer.proto`:

> [ZcashFoundation/zebra](https://github.com/ZcashFoundation/zebra/blob/main/zebra-rpc/proto/indexer.proto)

## Methods

| Method | Type | Purpose |
|--------|------|---------|
| `ChainTipChange` | streaming | Push (hash, height) on every chain tip change |
| `NonFinalizedStateChange` | streaming | Stream blocks in non-finalized state |
| `MempoolChange` | streaming | Push tx added/invalidated/mined events |
| `GetBlock` | unary | Fetch a block by 32-byte hash or 4-byte height |

## License

MIT OR Apache-2.0, matching Zebra's license.