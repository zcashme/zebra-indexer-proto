//! Rust bindings for Zebra's indexer gRPC interface.
//!
//! Generated from `indexer.proto` (vendored from Zebra).
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
//! How Zebra's server behaves where the proto is silent, as of Zebra
//! v7.0.0 (`zebra-rpc/src/indexer/methods.rs`).
//!
//! - `chain_tip_change` sends the current tip at once if Zebra's tip has
//!   changed since the indexer server started. Each subscription waits on a
//!   clone of the server's one `LatestChainTip` (lines 53 and 57), and a
//!   cloned watch receiver keeps the original's last-seen version.
//! - `non_finalized_state_change` rejects more than
//!   [`MAX_CHAIN_TIP_HASHES`] hashes, or any hash that is not 32 bytes,
//!   with `invalid_argument` (lines 302 and 319).
//! - A stream ends with `unavailable` when Zebra's internal channel closes
//!   or the subscription fails (lines 89, 127, 168, 234).
//! - A stream ends cleanly, with no status, when the consumer leaves a
//!   message unsent for 60 seconds behind a 64-message buffer (lines 23,
//!   31, 75, 154, 219).
//!

pub mod zebra {
    pub mod indexer {
        pub mod rpc {
            include!("../proto/__generated__/zebra.indexer.rpc.rs");
        }
    }
}

pub use zebra::indexer::rpc::*;

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
    /// Zebra may report a kind this version of the crate predates.
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
    /// A tip whose hash is given in display order.
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
    /// A consensus-serialized block and its hash in display order.
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

impl From<HashOrHeight> for BlockRequest {
    /// A hash is its 32 bytes; a height is 4 big-endian bytes.
    fn from(block: HashOrHeight) -> Self {
        let hash_or_height = match block {
            HashOrHeight::Hash(hash) => hash.to_vec(),
            HashOrHeight::Height(height) => height.to_be_bytes().to_vec(),
        };
        Self { hash_or_height }
    }
}

impl BlockRequest {
    /// Returns the block this request names, or `None` if the field is
    /// neither 32 nor 4 bytes.
    pub fn hash_or_height(&self) -> Option<HashOrHeight> {
        match self.hash_or_height.len() {
            32 => hash32(&self.hash_or_height).map(|hash| HashOrHeight::Hash(*hash)),
            4 => self
                .hash_or_height
                .as_slice()
                .try_into()
                .ok()
                .map(|height| HashOrHeight::Height(u32::from_be_bytes(height))),
            _ => None,
        }
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
        let chain_tip_hashes: Vec<Vec<u8>> = tips.into_iter().map(|tip| tip.to_vec()).collect();
        (chain_tip_hashes.len() <= MAX_CHAIN_TIP_HASHES).then_some(Self { chain_tip_hashes })
    }

    /// Returns this request's chain tip hashes in display order, or `None` if
    /// any hash is not 32 bytes. The count is checked separately against
    /// [`MAX_CHAIN_TIP_HASHES`].
    pub fn chain_tip_hashes_display_order(&self) -> Option<impl Iterator<Item = &[u8; 32]> + '_> {
        let all_valid = self
            .chain_tip_hashes
            .iter()
            .all(|hash| hash32(hash).is_some());

        all_valid.then(|| self.chain_tip_hashes.iter().filter_map(|hash| hash32(hash)))
    }
}

pub use tonic::transport::Channel;

pub type ZebraClient = zebra::indexer::rpc::indexer_client::IndexerClient<Channel>;

pub use tonic::transport::Error as ConnectError;

pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!("../proto/__generated__/indexer_descriptor.bin");

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
        let by_height = BlockRequest::from(HashOrHeight::Height(1000));
        assert_eq!(by_height.hash_or_height, [0, 0, 3, 232]);
        assert_eq!(by_height.hash_or_height(), Some(HashOrHeight::Height(1000)));

        let by_hash = BlockRequest::from(HashOrHeight::Hash([5; 32]));
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

        let legacy = MempoolChangeMessage::new(MempoolChangeKind::Added, [1; 32], None);
        assert!(legacy.auth_digest.is_empty());
        assert_eq!(legacy.auth_digest_display_order(), None);
    }

    #[test]
    fn a_resume_request_holds_at_most_the_limit() {
        let full = NonFinalizedStateChangeRequest::new(vec![[3; 32]; MAX_CHAIN_TIP_HASHES])
            .expect("the limit itself is accepted");
        assert_eq!(
            full.chain_tip_hashes_display_order()
                .map(|tips| tips.count()),
            Some(MAX_CHAIN_TIP_HASHES)
        );
        assert_eq!(
            NonFinalizedStateChangeRequest::new(vec![[3; 32]; MAX_CHAIN_TIP_HASHES + 1]),
            None
        );
    }
}
