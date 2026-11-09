# zebra-indexer-proto

Rust bindings for Zebra's indexer gRPC interface — the small set of streaming
and block-fetch RPCs Zebra serves on its indexer endpoint (default
`127.0.0.1:8230`).

## Why this crate exists

Zebra defines the interface in `zebra-rpc/proto/indexer.proto` and generates
its own bindings internally. This crate vendors it, generates the bindings once, and commits
them.

## Quick start

```toml
[dependencies]
zebra-indexer-proto = "3.0"
```

`ZebraClient` pins the transport for you without having to keep a 'tonic' dependency.

### Client

```rust
use zebra_indexer_proto::{ZebraClient, Empty};

let mut client = ZebraClient::connect("http://127.0.0.1:8230").await?;

// Chain tip change stream
let mut stream = client.chain_tip_change(Empty {}).await?.into_inner();
while let Some(tip) = stream.message().await? {
    let hash = tip.hash_display_order().expect("32 bytes");
    println!("tip: height={} hash={}", tip.height, hex::encode(hash));
}
```

### Field rules

Byte fields carry rules prost can't type. The crate encodes and decodes them:

```rust
use zebra_indexer_proto::{BlockRequest, HashOrHeight};

// A height is 4 big-endian bytes; a hash is 32 bytes in display order.
let request = BlockRequest::from(HashOrHeight::Height(1000));
let block = client.get_block(request).await?.into_inner();
let hash: &[u8; 32] = block.hash_display_order().expect("32 bytes");
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
        Some(MempoolChangeKind::Added) => { /* added to the mempool */ }
        Some(MempoolChangeKind::Invalidated) => { /* invalidated and rejected from the mempool */ }
        Some(MempoolChangeKind::Mined) => { /* mined into a block on the best chain */ }
        None => { /* unknown value from a newer Zebra */ }
    }
    // Display order, as Zebra v7.0.0 sends it.
    let tx_hash: &[u8; 32] = change.tx_hash_display_order().expect("32 bytes");
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
| `ChainTipChange` | stream | notifies listeners of chain tip changes |
| `NonFinalizedStateChange` | stream | notifies listeners of new blocks in the non-finalized state, after the chain tips the caller already has |
| `MempoolChange` | stream | notifies listeners of mempool changes |
| `GetBlock` | unary | returns the block with the given hash or height from the best chain |

## Staying in sync with Zebra

`proto/indexer.proto` is a byte-level copy of
[zebra-rpc/proto/indexer.proto](https://github.com/ZcashFoundation/zebra/blob/main/zebra-rpc/proto/indexer.proto)
at a tagged Zebra release, currently v7.0.0. The Zebra server facts in the
crate docs cite that release too. CI diffs the proto against Zebra's `main`
daily, so an upstream change surfaces before the next release.

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
