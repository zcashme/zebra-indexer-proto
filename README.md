# zebra-indexer-proto

Rust bindings for Zebra's two gRPC services:

- the **Indexer**, the small set of streaming and block-fetch RPCs Zebra
  serves on its indexer endpoint (`indexer_listen_addr`), at the crate root
- the lightwalletd **`CompactTxStreamer`**, which zebrad serves since v6.4.0
  when `lightwalletd_listen_addr` is set, in the `lightwalletd` module

## Why this crate exists

Zebra defines the services in `zebra-rpc/proto/` (`indexer.proto`,
`service.proto` and `compact_formats.proto`) and generates its own bindings
internally. This crate vendors those files, generates the bindings once, and
commits them, so a client of zebrad needs neither `protoc` nor `zebra-rpc`.

## Quick start

```toml
[dependencies]
zebra-indexer-proto = "3.1"
```

`ZebraClient` and `LightwalletdClient` pin the transport for you without having to keep a 'tonic' dependency.

### Client

```rust
use zebra_indexer_proto::{ZebraClient, Empty};

let mut client = ZebraClient::connect("http://127.0.0.1:8230").await?;

// Chain tip change stream
let mut stream = client.chain_tip_change(Empty {}).await?.into_inner();
while let Some(tip) = stream.message().await? {
    let hash: &[u8; 32] = tip.hash_display_order().expect("32 bytes");
    println!("tip: height={} hash={:02x?}", tip.height, hash);
}
```

### lightwalletd client

```rust
use zebra_indexer_proto::{lightwalletd::{ChainSpec, TxFilter}, LightwalletdClient, TxLocation};

let mut client = LightwalletdClient::connect("http://127.0.0.1:9067").await?;

let tip = client.get_latest_block(ChainSpec {}).await?.into_inner();
// BlockID hashes are in internal byte order, the reverse of display order.
let hash: &[u8; 32] = tip.hash_internal_order().expect("32 bytes");

let tx = client
    .get_transaction(TxFilter::from_txid_internal_order(txid))
    .await?
    .into_inner();
match tx.location() {
    Some(TxLocation::Mempool) => { /* in the mempool */ }
    Some(TxLocation::OtherChain) => { /* mined on a fork */ }
    Some(TxLocation::BestChain(height)) => { /* mined at `height` */ }
    None => { /* a height Zebra never sends */ }
}
```

Which order each `CompactTxStreamer` hash uses, as Zebra v7.0.0 sends it:

| Field | Order | Helper |
|---|---|---|
| `BlockID.hash` | internal | `BlockId::from_hash_internal_order`, `hash_internal_order()` |
| `TxFilter.hash` | internal | `TxFilter::from_txid_internal_order` |
| `CompactBlock.hash`, `prevHash`, `CompactTx.hash`, `GetAddressUtxosReply.txid` | internal | |
| `SubtreeRoot.completingBlockHash` | display | `completing_block_hash_display_order()` |
| `TreeState.hash`, `SendResponse` txid | display hex string | `SendResponse::result()` |

These are zebrad's orders. lightwalletd itself still reads `GetTreeState`'s
`BlockID.hash` in display order (`zebra-rpc/src/lightwalletd/methods.rs:608–610`).

Heights in a `BlockID` have two readings. `GetBlock`, the block ranges and
`GetTreeState` reject height 0 with no hash, so `BlockId::from_height(0)` is
`None`. A `TransparentAddressBlockFilter` range reads a missing or zero
height as an open end. Ranges never take a hash.

### Field rules

Byte fields carry rules prost can't type. The crate encodes and decodes them:

```rust
use zebra_indexer_proto::{BlockRequest, HashOrHeight};

// A height is 4 big-endian bytes; a hash is 32 bytes in display order.
// `new` refuses a height above MAX_BLOCK_HEIGHT, which Zebra rejects.
let request = BlockRequest::new(HashOrHeight::Height(1000)).expect("a height Zebra serves");
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

`CompactTxStreamer` (`lightwalletd` module), from `service.proto`:

| RPC | Type | What it does |
|---|---|---|
| `GetLatestBlock` | unary | the best chain tip's height and hash |
| `GetBlock` | unary | a compact block by height or hash |
| `GetBlockNullifiers` | unary | a compact block with only nullifiers |
| `GetBlockRange` | stream | compact blocks in a height range |
| `GetBlockRangeNullifiers` | stream | compact blocks in a range, only nullifiers |
| `GetTransaction` | unary | a full transaction by txid, with where it is |
| `SendTransaction` | unary | submits a transaction |
| `GetTaddressTxids` | stream | deprecated, same as `GetTaddressTransactions` |
| `GetTaddressTransactions` | stream | full transactions for a transparent address in a block range |
| `GetTaddressBalance` | unary | the balance of a list of transparent addresses |
| `GetTaddressBalanceStream` | client stream | the same, addresses streamed in |
| `GetMempoolTx` | stream | compact mempool transactions, excluding given txids |
| `GetMempoolStream` | stream | full mempool transactions, closing when a new block is mined |
| `GetTreeState` | unary | the note commitment trees at a block |
| `GetLatestTreeState` | unary | the note commitment trees at the tip |
| `GetSubtreeRoots` | stream | note commitment subtree roots for a pool |
| `GetAddressUtxos` | unary | UTXOs for transparent addresses |
| `GetAddressUtxosStream` | stream | the same, streamed |
| `GetLightdInfo` | unary | information about the server and chain |
| `Ping` | unary | testing only |

## Staying in sync with Zebra

`proto/` is a byte-level copy of
[zebra-rpc/proto](https://github.com/ZcashFoundation/zebra/tree/main/zebra-rpc/proto)
at a tagged Zebra release, currently v7.0.0. The Zebra server facts in the
crate docs cite that release too. CI compares the whole directory with the
release named by `ZEBRA_TAG` in `.github/workflows/ci.yml`, so a proto Zebra
adds also fails it.

To update the bindings:

```bash
cp ../zebra/zebra-rpc/proto/*.proto proto/
cargo build --features regenerate   # needs protoc; maintainers only
```

Copy from the Zebra release you are moving to, set `ZEBRA_TAG` to it, and
recheck each Zebra server fact's citation. Then commit the protos
and everything under `proto/__generated__/`, bump the version, and publish. Normal builds never need the `regenerate`
feature — it exists solely for this workflow.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

MIT OR Apache-2.0 — same as Zebra.
