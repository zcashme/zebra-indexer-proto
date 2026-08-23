# zebra-indexer-proto

Rust bindings for Zebra's indexer gRPC interface — the small set of streaming
and block-fetch RPCs Zebra serves on its indexer endpoint (default
`127.0.0.1:8230`).

## Why this crate exists

Zebra defines the interface in `zebra-rpc/proto/indexer.proto` and generates
its own bindings internally. This crate vendors it, generates the bindings once, and commits
them. `

## Quick start

```toml
[dependencies]
zebra-indexer-proto = "2.4"
```

`ZebraClient` pins the transport for you without having to keep a 'tonic' dependency.

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

### Implementing a server

The crate re-exports the server trait verbatim from the generated bindings i.e. the `Indexer` trait Zebra implements internally. Fill in the four
generated methods exactly as Zebra does.

```rust
use zebra_indexer_proto::indexer_server::{Indexer, IndexerServer};

#[tonic::async_trait]
impl Indexer for MyService {
    type ChainTipChangeStream = /* ... */;
    // the four methods ...
}

let server = IndexerServer::new(my_service);
```

### Serde

Every generated message derives `serde::Serialize` and `serde::Deserialize`:

```rust
let json = serde_json::to_string(&tip)?;            // {"hash":[1,1,...],"height":123456}
let back: BlockHashAndHeight = serde_json::from_str(&json)?;
```

### Reflection

```rust
use zebra_indexer_proto::FILE_DESCRIPTOR_SET;

let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build_v1()
    .unwrap();
```

## The interface

| RPC | Type | What it does |
|---|---|---|
| `ChainTipChange` | stream | push `(hash, height)` on every best-chain tip change |
| `NonFinalizedStateChange` | stream | stream blocks in non-finalized state, resumable via known tip hashes |
| `MempoolChange` | stream | notify on tx added / invalidated / mined |
| `GetBlock` | unary | fetch a block by hash (32 bytes, display order) or height (4 bytes, big-endian) |

## Staying in sync with Zebra

`proto/indexer.proto` is a copy of
[zebra-rpc/proto/indexer.proto](https://github.com/ZcashFoundation/zebra/blob/main/zebra-rpc/proto/indexer.proto).
CI diffs the two daily and fails if they drift, so upstream changes surface
here quickly.

To update the bindings:

```bash
cp ../zebra/zebra-rpc/proto/indexer.proto proto/indexer.proto
cargo build --features regenerate   # needs protoc; maintainers only
```

Then commit `proto/indexer.proto` and everything under `proto/__generated__/`,
bump the version, and publish. Normal builds never need the `regenerate`
feature — it exists solely for this workflow.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

MIT OR Apache-2.0 — same as Zebra.
