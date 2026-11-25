//! Rust bindings for Zebra's two gRPC services: the Indexer, and the
//! lightwalletd `CompactTxStreamer`.
//!
//! Generated from `indexer.proto`, `service.proto` and
//! `compact_formats.proto` (vendored from Zebra). The Indexer's types are at
//! the crate root; the `CompactTxStreamer`'s are in [`lightwalletd`], since
//! both define `Empty`.
//!
//! The generated code is **committed** under `proto/__generated__/`.
//! A normal `cargo build` (or `cargo add zebra-indexer-proto`) has no
//! `protoc` requirement and pulls in no code-generation build dependencies.
//!
//! Maintainers can regenerate with:
//!     cargo build --features regenerate
//!
//! # Zebra server facts
//!
//! How Zebra's Indexer server behaves where the proto is silent, as of Zebra
//! v7.0.0 (`zebra-rpc/src/indexer/methods.rs`). The `CompactTxStreamer`'s
//! are in [`lightwalletd`].
//!
//! - `chain_tip_change` sends the current tip at once whenever Zebra has a
//!   tip. Each subscription waits on a clone of the server's one
//!   `LatestChainTip` (lines 53 and 57), whose watch receiver starts with
//!   the tip unseen (`zebra-state/src/service/chain_tip.rs:161–172`).
//! - `non_finalized_state_change` rejects more than
//!   [`MAX_CHAIN_TIP_HASHES`] hashes, or any hash that is not 32 bytes,
//!   with `invalid_argument` (lines 302 and 319).
//! - `get_block` rejects a height above [`MAX_BLOCK_HEIGHT`] with
//!   `invalid_argument` (line 258).
//! - A stream ends with `unavailable` when Zebra's internal channel closes,
//!   the subscription fails, or a `mempool_change` consumer falls more than
//!   20 changes behind Zebra's broadcast (lines 89, 127, 168, 188, 234).
//! - A stream ends cleanly, with no status, when the consumer leaves a
//!   message unsent for 60 seconds behind a 64-message buffer (lines 23,
//!   31, 75, 154, 219).
//! - Zebra's own client resubscribes after any stream end, clean or not
//!   (`zebra-rpc/src/sync.rs:147–160, 319–334`).
//!

pub mod zebra {
    pub mod indexer {
        pub mod rpc {
            include!("../proto/__generated__/zebra.indexer.rpc.rs");
        }
    }
}

pub use zebra::indexer::rpc::*;

/// Bindings for the lightwalletd `CompactTxStreamer` service, package
/// `cash.z.wallet.sdk.rpc`, as Zebra serves it.
///
/// # Zebra server facts
///
/// How Zebra's `CompactTxStreamer` server behaves where the proto is silent,
/// as of Zebra v7.0.0 (`zebra-rpc/src/lightwalletd/methods.rs` unless
/// named).
///
/// - zebrad starts the service only when `[rpc] lightwalletd_listen_addr` is
///   set (`zebrad/src/commands/start.rs:577`), and serves gRPC reflection
///   beside it (`zebra-rpc/src/lightwalletd/server.rs:80, 100–101`).
/// - Hashes in `BlockID`, `TxFilter`, `CompactBlock` (`hash`, `prevHash`),
///   `CompactTx.hash` and `GetAddressUtxosReply.txid` are in internal byte
///   order (lines 92, 607–615, 161, 980; `zebra-rpc/src/lightwalletd.rs:115–116,
///   175`). `SubtreeRoot.completingBlockHash` is in display order, as
///   lightwalletd sends it (lines 526–528). `TreeState.hash` and
///   `SendResponse`'s txid are display-order hex strings (lines 1136, 241).
/// - A `BlockID` with no hash and height 0 is `invalid_argument` in
///   `GetBlock`, `GetBlockNullifiers`, `GetBlockRange`,
///   `GetBlockRangeNullifiers` and `GetTreeState` (lines 616–622). In a
///   `TransparentAddressBlockFilter` range, a missing or zero height is an
///   open end instead (lines 818–831). Both kinds of range reject a hash
///   with `invalid_argument` (lines 764–766, 823–826).
/// - `GetSubtreeRoots` rejects a `startIndex` above `u16::MAX` with
///   `invalid_argument`, caps `maxEntries` at `u16::MAX`, and reads a
///   `maxEntries` of 0 as no limit (lines 460–471).
/// - `GetTreeState`, `GetLatestTreeState`, `GetSubtreeRoots` and
///   `GetLightdInfo` return a JSON-RPC `-5` or `-8` as `not_found`, any
///   other error as `internal` (lines 422, 442, 477, 569–570, 1151–1157).
///   The transparent address calls (`GetTaddressTxids`,
///   `GetTaddressTransactions`, `GetTaddressBalance`,
///   `GetTaddressBalanceStream`, `GetAddressUtxos`, `GetAddressUtxosStream`)
///   return `-5`, `-8` or `-32602` as `invalid_argument` instead (lines 841,
///   945, 970, 1166–1171).
/// - `GetTransaction` looks in the best chain, then the mempool, then side
///   chains, and answers `not_found` when the transaction is in none (lines
///   164–216).
/// - `SendTransaction` returns submission errors in the `SendResponse`, with
///   Zebra's JSON-RPC code and message, not as a gRPC status. The one
///   exception is a transaction above [`MAX_RAW_TRANSACTION_BYTES`], which
///   gets `invalid_argument` before submission (line 228). On success
///   `errorCode` is 0 and `errorMessage` is the transaction id as display
///   hex (lines 236–247).
/// - `GetMempoolStream` first sends every transaction already in the
///   mempool, then each one added (line 354), and ends when the best tip
///   changes (line 362). A subscriber that reopens it after each block
///   receives the transactions still pending again.
/// - `GetTreeState` answers from `z_gettreestate` (line 420). A pool that is
///   not active yet has an empty tree string, as `ironwoodTree` before NU6.3
///   (lines 1140–1145).
pub mod lightwalletd {
    include!("../proto/__generated__/cash.z.wallet.sdk.rpc.rs");
}

// ============================================================================
// Typed accessors
//
// The generated types are raw where the protocol has meaning: prost leaves
// enum fields as `i32` and byte fields as `Vec<u8>`, with the conventions
// (variant numbering, hash byte order) carrying no compiler-enforced
// correspondence to their meaning. The thin layer below closes that gap
// without duplicating it: it re-uses the enumeration prost generates from
// the proto, so renumbering the wire enum breaks this crate's build rather
// than silently mislabeling a change downstream.
// ============================================================================

/// The kind of mempool change reported by a [`MempoolChangeMessage`].
///
/// This is the `ChangeType` enumeration prost generates from the proto,
/// re-exported under the name Zebra's own stack uses for the same taxonomy
/// (the `MempoolChangeKind` in `zebra-node-services`), so downstream crates
/// can share the vocabulary without the Zebra dependency.
///
/// [`Default`] is [`MempoolChangeKind::Added`], the proto3 zero value.
pub use zebra::indexer::rpc::mempool_change_message::ChangeType as MempoolChangeKind;

fn hash32(bytes: &[u8]) -> Option<&[u8; 32]> {
    bytes.try_into().ok()
}

impl MempoolChangeMessage {
    /// A change to the transaction whose hashes are given in display order.
    /// No auth digest is sent as an empty field.
    pub fn new(kind: MempoolChangeKind, tx_hash: [u8; 32], auth_digest: Option<[u8; 32]>) -> Self {
        Self {
            change_type: kind.into(),
            tx_hash: tx_hash.to_vec(),
            auth_digest: auth_digest
                .map(|digest| digest.to_vec())
                .unwrap_or_default(),
        }
    }

    /// Returns the kind of change, or `None` if the wire value is unknown.
    ///
    /// Unknown values are representable: proto3 enums are open, and a newer
    /// Zebra may report a kind this version of the crate predates. The
    /// generated `change_type()` returns `Added` for an unknown value.
    pub fn kind(&self) -> Option<MempoolChangeKind> {
        MempoolChangeKind::try_from(self.change_type).ok()
    }

    /// Returns the transaction hash, or `None` if the field is not 32 bytes.
    ///
    /// Zebra v7.0.0 sends the transaction's mined id in display order
    /// (`zebra-rpc/src/indexer/methods.rs:200`).
    pub fn tx_hash_display_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.tx_hash)
    }

    /// Returns the auth digest, or `None` if the field is not 32 bytes.
    ///
    /// Zebra v7.0.0 sends it in display order, and sends an empty field for a
    /// transaction without one (`zebra-rpc/src/indexer/methods.rs:201–203`).
    pub fn auth_digest_display_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.auth_digest)
    }
}

impl BlockHashAndHeight {
    /// A block hash, in display order, and its height.
    pub fn new(hash: [u8; 32], height: u32) -> Self {
        Self {
            hash: hash.to_vec(),
            height,
        }
    }

    /// Returns the block hash in display order, or `None` if the field is not
    /// 32 bytes.
    pub fn hash_display_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.hash)
    }
}

impl BlockAndHash {
    /// An encoded block and its hash, in display order.
    pub fn new(hash: [u8; 32], data: Vec<u8>) -> Self {
        Self {
            hash: hash.to_vec(),
            data,
        }
    }

    /// Returns the block hash in display order, or `None` if the field is not
    /// 32 bytes.
    pub fn hash_display_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.hash)
    }
}

/// The block a [`BlockRequest`] names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashOrHeight {
    /// A block hash in display order.
    Hash([u8; 32]),
    /// A block height.
    Height(u32),
}

/// The highest block height Zebra v7.0.0 serves: `block::Height::MAX`
/// (`zebra-chain/src/block/height.rs:67`), checked by `get_block`
/// (`zebra-rpc/src/indexer/methods.rs:258`).
pub const MAX_BLOCK_HEIGHT: u32 = u32::MAX / 2;

impl BlockRequest {
    /// A request for `block`: a hash is its 32 bytes, a height is 4
    /// big-endian bytes. `None` for a height above [`MAX_BLOCK_HEIGHT`].
    pub fn new(block: HashOrHeight) -> Option<Self> {
        let hash_or_height = match block {
            HashOrHeight::Hash(hash) => hash.to_vec(),
            HashOrHeight::Height(height) if height <= MAX_BLOCK_HEIGHT => {
                height.to_be_bytes().to_vec()
            }
            HashOrHeight::Height(_) => return None,
        };
        Some(Self { hash_or_height })
    }

    /// Returns the block this request names, or `None` if the field is
    /// neither 32 nor 4 bytes, or names a height above [`MAX_BLOCK_HEIGHT`].
    pub fn hash_or_height(&self) -> Option<HashOrHeight> {
        let bytes = self.hash_or_height.as_slice();
        if let Ok(hash) = <[u8; 32]>::try_from(bytes) {
            return Some(HashOrHeight::Hash(hash));
        }
        let height = u32::from_be_bytes(bytes.try_into().ok()?);
        (height <= MAX_BLOCK_HEIGHT).then_some(HashOrHeight::Height(height))
    }
}

/// The maximum number of chain tip hashes a [`NonFinalizedStateChangeRequest`]
/// may carry.
///
/// Zebra v7.0.0 rejects a larger request with `invalid_argument` on every
/// network (`zebra-rpc/src/indexer/methods.rs:302`), against
/// `MAX_NON_FINALIZED_CHAIN_FORKS_TEST_NETWORKS`
/// (`zebra-state/src/constants.rs:113`).
pub const MAX_CHAIN_TIP_HASHES: usize = 100;

impl NonFinalizedStateChangeRequest {
    /// A request carrying the given chain tip hashes in display order, or
    /// `None` if there are more than [`MAX_CHAIN_TIP_HASHES`].
    pub fn new(tips: impl IntoIterator<Item = [u8; 32]>) -> Option<Self> {
        let chain_tip_hashes: Vec<Vec<u8>> = tips
            .into_iter()
            .take(MAX_CHAIN_TIP_HASHES + 1)
            .map(|tip| tip.to_vec())
            .collect();
        (chain_tip_hashes.len() <= MAX_CHAIN_TIP_HASHES).then_some(Self { chain_tip_hashes })
    }

    /// Returns this request's chain tip hashes in display order, or `None` if
    /// there are more than [`MAX_CHAIN_TIP_HASHES`] or any hash is not 32
    /// bytes. Zebra checks in that order.
    pub fn chain_tip_hashes_display_order(&self) -> Option<impl Iterator<Item = &[u8; 32]> + '_> {
        if self.chain_tip_hashes.len() > MAX_CHAIN_TIP_HASHES {
            return None;
        }
        let tips: Vec<&[u8; 32]> = self
            .chain_tip_hashes
            .iter()
            .map(|hash| hash32(hash))
            .collect::<Option<_>>()?;
        Some(tips.into_iter())
    }
}

// ============================================================================
// lightwalletd field rules
//
// The same gap as the Indexer's: a hash's byte order, a height's sentinel
// values, a field's range and an open enum are rules prost can't type.
// Most `CompactTxStreamer` hashes are in internal (little-endian) byte
// order, the reverse of the display order the Indexer uses; the one byte
// field in display order is `SubtreeRoot.completing_block_hash`.
// ============================================================================

/// The largest transaction Zebra v7.0.0 accepts in `SendTransaction`:
/// `block::MAX_BLOCK_BYTES` (`zebra-chain/src/block/serialize.rs:24`),
/// checked before submission with `invalid_argument`
/// (`zebra-rpc/src/lightwalletd/methods.rs:51` and 228).
pub const MAX_RAW_TRANSACTION_BYTES: usize = 2_000_000;

impl lightwalletd::BlockId {
    /// A block by height, for `GetBlock`, `GetBlockNullifiers`,
    /// `GetBlockRange`, `GetBlockRangeNullifiers` and `GetTreeState`. `None`
    /// for 0, which those calls read as neither field set and reject with
    /// `invalid_argument`, and for a height above [`MAX_BLOCK_HEIGHT`]
    /// (`zebra-rpc/src/lightwalletd/methods.rs:616–629`). `GetBlock`,
    /// `GetBlockNullifiers` and `GetTreeState` reach genesis only by hash;
    /// the block ranges take no hash, so they can't return genesis.
    ///
    /// A `TransparentAddressBlockFilter` range reads height 0 differently:
    /// as an open end (lines 818–831). Leave `start` or `end` as `None`, or
    /// use `BlockId::default()`, for the full range.
    pub fn from_height(height: u32) -> Option<Self> {
        (height != 0 && height <= MAX_BLOCK_HEIGHT).then(|| Self {
            height: height.into(),
            hash: Vec::new(),
        })
    }

    /// A block by hash, in internal byte order, for `GetBlock`,
    /// `GetBlockNullifiers` and `GetTreeState`. Zebra reads a non-empty hash
    /// before the height (`zebra-rpc/src/lightwalletd/methods.rs:607–615`).
    ///
    /// Ranges take heights only: a hash in a `BlockRange` or a
    /// `TransparentAddressBlockFilter` range is `invalid_argument` (lines
    /// 764–766 and 823–826).
    pub fn from_hash_internal_order(hash: [u8; 32]) -> Self {
        Self {
            height: 0,
            hash: hash.to_vec(),
        }
    }

    /// Returns the block hash in internal byte order, or `None` if the field
    /// is not 32 bytes. Zebra v7.0.0's `GetLatestBlock` sends it so
    /// (`zebra-rpc/src/lightwalletd/methods.rs:92`).
    pub fn hash_internal_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.hash)
    }
}

impl lightwalletd::TxFilter {
    /// A lookup by transaction id, in internal byte order: the only
    /// specification Zebra supports (`zebra-rpc/src/lightwalletd/methods.rs:154–161`).
    pub fn from_txid_internal_order(txid: [u8; 32]) -> Self {
        Self {
            hash: txid.to_vec(),
            ..Default::default()
        }
    }
}

/// Where a [`lightwalletd::RawTransaction`] is, from its `height` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxLocation {
    /// In the mempool: height 0. The genesis block's height is also 0, and
    /// Zebra sends it for the genesis coinbase
    /// (`zebra-rpc/src/lightwalletd/methods.rs:173`), so that one mined
    /// transaction reads as `Mempool` too.
    Mempool,
    /// Mined on a fork that is not the best chain: height `u64::MAX`.
    OtherChain,
    /// Mined in the best chain at this height.
    BestChain(u32),
}

impl lightwalletd::RawTransaction {
    /// A transaction to submit. `None` above [`MAX_RAW_TRANSACTION_BYTES`].
    pub fn new(data: Vec<u8>) -> Option<Self> {
        (data.len() <= MAX_RAW_TRANSACTION_BYTES).then_some(Self { data, height: 0 })
    }

    /// Returns where the transaction is, or `None` for a height that is
    /// neither sentinel and above [`MAX_BLOCK_HEIGHT`].
    ///
    /// The proto defines the sentinels. Zebra v7.0.0 sets them in
    /// `GetTransaction` (`zebra-rpc/src/lightwalletd/methods.rs:206` and 213)
    /// and sends height 0 for every `GetMempoolStream` transaction (line
    /// 1125), where the proto's comment says the latest block height. Height
    /// 0 is ambiguous on the wire: the genesis coinbase is mined at 0, so
    /// `GetTransaction` for it also reads as [`TxLocation::Mempool`].
    pub fn location(&self) -> Option<TxLocation> {
        match self.height {
            0 => Some(TxLocation::Mempool),
            u64::MAX => Some(TxLocation::OtherChain),
            height => u32::try_from(height)
                .ok()
                .filter(|height| *height <= MAX_BLOCK_HEIGHT)
                .map(TxLocation::BestChain),
        }
    }
}

impl lightwalletd::SendResponse {
    /// `Ok` with the message when `errorCode` is zero, which the proto
    /// defines as success; otherwise `Err` with the code and message.
    pub fn result(&self) -> Result<&str, (i32, &str)> {
        match self.error_code {
            0 => Ok(&self.error_message),
            code => Err((code, &self.error_message)),
        }
    }
}

impl lightwalletd::GetSubtreeRootsArg {
    /// A request for `protocol`'s subtree roots from `start_index`. Zebra
    /// rejects a start index above `u16::MAX` with `invalid_argument` and
    /// caps `max_entries` at `u16::MAX` (`zebra-rpc/src/lightwalletd/methods.rs:460–471`),
    /// so both are `u16` here. A `max_entries` of 0 means no limit.
    pub fn new(
        protocol: lightwalletd::ShieldedProtocol,
        start_index: u16,
        max_entries: u16,
    ) -> Self {
        Self {
            start_index: start_index.into(),
            shielded_protocol: protocol.into(),
            max_entries: max_entries.into(),
        }
    }

    /// Returns the pool, or `None` if the wire value is unknown. The
    /// generated `shielded_protocol()` returns `Sapling` for an unknown
    /// value.
    pub fn protocol(&self) -> Option<lightwalletd::ShieldedProtocol> {
        lightwalletd::ShieldedProtocol::try_from(self.shielded_protocol).ok()
    }
}

impl lightwalletd::SubtreeRoot {
    /// Returns the hash of the block that completed the subtree, in display
    /// order, or `None` if the field is not 32 bytes. Unlike the other
    /// `CompactTxStreamer` hashes, Zebra v7.0.0 sends this one in display
    /// order, as lightwalletd does (`zebra-rpc/src/lightwalletd/methods.rs:526–528`).
    pub fn completing_block_hash_display_order(&self) -> Option<&[u8; 32]> {
        hash32(&self.completing_block_hash)
    }
}

pub use tonic::transport::Channel;

pub type ZebraClient = zebra::indexer::rpc::indexer_client::IndexerClient<Channel>;

pub use tonic::transport::Error as ConnectError;

pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!("../proto/__generated__/indexer_descriptor.bin");

pub type LightwalletdClient =
    lightwalletd::compact_tx_streamer_client::CompactTxStreamerClient<Channel>;

pub const LIGHTWALLETD_FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!("../proto/__generated__/lightwalletd_descriptor.bin");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_usable() {
        let _empty = Empty {};
        let tip = BlockHashAndHeight {
            hash: vec![1; 32],
            height: 123456,
        };
        assert_eq!(tip.height, 123456);
    }

    #[test]
    fn client_and_server_types_exist() {
        use zebra::indexer::rpc::indexer_client::IndexerClient;
        use zebra::indexer::rpc::indexer_server::IndexerServer;

        fn _assert_client(_: IndexerClient<tonic::transport::Channel>) {}
        let _ = std::any::type_name::<IndexerServer<()>>();
        let _name = std::any::type_name::<fn() -> IndexerServer<()>>();
    }

    #[test]
    fn descriptor_is_non_empty() {
        assert!(!FILE_DESCRIPTOR_SET.is_empty());
        assert!(FILE_DESCRIPTOR_SET.len() > 100);
    }

    #[test]
    fn mempool_change_kind_matches_the_wire() {
        for (wire, expected) in [
            (0, Some(MempoolChangeKind::Added)),
            (1, Some(MempoolChangeKind::Invalidated)),
            (2, Some(MempoolChangeKind::Mined)),
            (3, None),
            (-1, None),
            (i32::MAX, None),
        ] {
            let msg = MempoolChangeMessage {
                change_type: wire,
                ..Default::default()
            };
            assert_eq!(msg.kind(), expected, "wire value {wire}");
            if let Some(kind) = expected {
                assert_eq!(i32::from(kind), wire);
            }
        }
    }

    #[test]
    fn display_order_accessors_require_32_bytes() {
        let mut msg = MempoolChangeMessage::default();
        assert_eq!(msg.tx_hash_display_order(), None);
        assert_eq!(msg.auth_digest_display_order(), None);

        msg.tx_hash = vec![0; 31];
        assert_eq!(msg.tx_hash_display_order(), None);

        msg.tx_hash = vec![0; 32];
        assert_eq!(msg.tx_hash_display_order(), Some(&[0; 32]));
        msg.auth_digest = vec![1; 32];
        assert_eq!(msg.auth_digest_display_order(), Some(&[1; 32]));

        let tip = BlockHashAndHeight {
            hash: vec![2; 32],
            height: 1,
        };
        assert_eq!(tip.hash_display_order(), Some(&[2; 32]));

        let block = BlockAndHash {
            hash: vec![4; 32],
            data: Vec::new(),
        };
        assert_eq!(block.hash_display_order(), Some(&[4; 32]));
        assert_eq!(BlockAndHash::default().hash_display_order(), None);
        assert_eq!(BlockHashAndHeight::default().hash_display_order(), None);
    }

    #[test]
    fn chain_tip_hash_accessors_match_server_validation() {
        let hash = || vec![3u8; 32];

        let req = NonFinalizedStateChangeRequest {
            chain_tip_hashes: vec![hash(), hash()],
        };
        let tips: Vec<&[u8; 32]> = req
            .chain_tip_hashes_display_order()
            .expect("32-byte hashes are valid")
            .collect();
        assert_eq!(tips, [&[3; 32], &[3; 32]]);

        // Empty is valid: it means "send everything in the non-finalized state".
        assert_eq!(
            NonFinalizedStateChangeRequest::default()
                .chain_tip_hashes_display_order()
                .map(|tips| tips.count()),
            Some(0)
        );

        // A wrong-length hash is rejected, matching the server's up-front check.
        let req = NonFinalizedStateChangeRequest {
            chain_tip_hashes: vec![hash(), vec![3; 31]],
        };
        assert!(req.chain_tip_hashes_display_order().is_none());
    }

    #[test]
    fn block_requests_round_trip_on_the_wire() {
        let by_height = BlockRequest::new(HashOrHeight::Height(1000)).unwrap();
        assert_eq!(by_height.hash_or_height, [0, 0, 3, 232]);
        assert_eq!(by_height.hash_or_height(), Some(HashOrHeight::Height(1000)));

        let by_hash = BlockRequest::new(HashOrHeight::Hash([5; 32])).unwrap();
        assert_eq!(by_hash.hash_or_height, [5; 32]);
        assert_eq!(by_hash.hash_or_height(), Some(HashOrHeight::Hash([5; 32])));

        for len in [0, 3, 5, 31, 33] {
            let request = BlockRequest {
                hash_or_height: vec![0; len],
            };
            assert_eq!(request.hash_or_height(), None, "{len} bytes");
        }
    }

    #[test]
    fn block_requests_stop_at_the_highest_height_zebra_serves() {
        let top = BlockRequest::new(HashOrHeight::Height(MAX_BLOCK_HEIGHT)).unwrap();
        assert_eq!(
            top.hash_or_height(),
            Some(HashOrHeight::Height(MAX_BLOCK_HEIGHT))
        );

        assert_eq!(
            BlockRequest::new(HashOrHeight::Height(MAX_BLOCK_HEIGHT + 1)),
            None
        );
        let above = BlockRequest {
            hash_or_height: (MAX_BLOCK_HEIGHT + 1).to_be_bytes().to_vec(),
        };
        assert_eq!(above.hash_or_height(), None);
    }

    #[test]
    fn constructors_round_trip_through_the_accessors() {
        let tip = BlockHashAndHeight::new([6; 32], 7);
        assert_eq!(tip.hash_display_order(), Some(&[6; 32]));
        assert_eq!(tip.height, 7);

        let block = BlockAndHash::new([8; 32], vec![9]);
        assert_eq!(block.hash_display_order(), Some(&[8; 32]));
        assert_eq!(block.data, [9]);

        let change = MempoolChangeMessage::new(MempoolChangeKind::Mined, [1; 32], Some([2; 32]));
        assert_eq!(change.kind(), Some(MempoolChangeKind::Mined));
        assert_eq!(change.tx_hash_display_order(), Some(&[1; 32]));
        assert_eq!(change.auth_digest_display_order(), Some(&[2; 32]));

        let no_digest = MempoolChangeMessage::new(MempoolChangeKind::Added, [1; 32], None);
        assert!(no_digest.auth_digest.is_empty());
        assert_eq!(no_digest.auth_digest_display_order(), None);
    }

    #[test]
    fn a_request_carries_at_most_max_chain_tip_hashes() {
        let full = NonFinalizedStateChangeRequest::new(vec![[3; 32]; MAX_CHAIN_TIP_HASHES])
            .expect("the limit itself is accepted");
        assert_eq!(
            full.chain_tip_hashes_display_order()
                .map(|tips| tips.count()),
            Some(MAX_CHAIN_TIP_HASHES)
        );
        assert_eq!(
            NonFinalizedStateChangeRequest::new(std::iter::repeat([3; 32])),
            None
        );

        let over = NonFinalizedStateChangeRequest {
            chain_tip_hashes: vec![vec![3; 32]; MAX_CHAIN_TIP_HASHES + 1],
        };
        assert!(over.chain_tip_hashes_display_order().is_none());
    }
}
