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
zebra-indexer-proto = "2.5"
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

### Typed mempool changes

`change_type` is a raw `i32` on the wire (prost's convention), but the crate
names its meaning for you. `kind()` returns `None` for values a newer Zebra
might add:

```rust
use zebra_indexer_proto::{Empty, MempoolChangeKind, ZebraClient};

let mut stream = client.mempool_change(Empty {}).await?.into_inner();
while let Some(change) = stream.message().await? {
    match change.kind() {
        Some(MempoolChangeKind::Added) => { /* tx entered the mempool */ }
        Some(MempoolChangeKind::Invalidated) => { /* pending observations withdrawn */ }
        Some(MempoolChangeKind::Mined) => { /* confirmed into a best-chain block */ }
        None => { /* unknown value from a newer Zebra */ }
    }
    // Display order, as with every other hash in this protocol.
    let tx_hash = change.tx_hash_display_order().expect("32 bytes");
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

`proto/indexer.proto` is a byte-level copy of
[zebra-rpc/proto/indexer.proto](https://github.com/ZcashFoundation/zebra/blob/main/zebra-rpc/proto/indexer.proto)
(Zebra vendored this crate's file for v6.3.0, so changes can land in either
place — whoever changes it opens the sync PR). CI diffs the two daily and
fails if they drift, so changes surface here quickly. When the change lands
here first, keep the diff open on the Zebra side until it syncs.

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
