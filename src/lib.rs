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
//! # Protocol behavior notes
//!
//! Facts about how Zebra's server behaves that the wire types alone don't
//! express. These are implementation choices, not wire format: they can
//! change in Zebra without a proto change.
//!
//! - `chain_tip_change` sends **no initial message**: a fresh subscriber
//!   doesn't learn the current tip until it *changes*. The protocol has no
//!   RPC to fetch the current tip, so a client that needs it at connect time
//!   must obtain it from elsewhere and treat the stream as change-only.
//! - `non_finalized_state_change` requests carrying more than
//!   [`MAX_CHAIN_TIP_HASHES`] hashes, or any hash that is not 32 bytes, are
//!   rejected up front with `invalid_argument`.
//! - Streams end with status `unavailable` when Zebra's internal channels
//!   close, and a consumer too slow to drain Zebra's per-subscriber buffer
//!   (64 messages, 60-second send timeout) has its stream dropped. A stream
//!   ending is a resubscribe signal, not a terminal error.
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

/// The length of a Zcash hash on the wire: block hashes and transaction IDs
/// are 32 bytes in display order.
const HASH_LEN: usize = 32;

impl MempoolChangeMessage {
    /// Returns the kind of change, or `None` if the wire value is unknown.
    ///
    /// Unknown values are representable: proto3 enums are open, and a newer
    /// Zebra may report a kind this version of the crate predates.
    pub fn kind(&self) -> Option<MempoolChangeKind> {
        MempoolChangeKind::try_from(self.change_type).ok()
    }

    /// Returns the transaction hash in display byte order, or `None` if the
    /// field is not 32 bytes.
    ///
    /// Zebra's indexer server sends the transaction's `mined_id` in display
    /// order — the same convention as every other hash in this protocol.
    /// Callers converting to `librustzcash` types reverse it into a
    /// transaction ID.
    pub fn tx_hash_display_order(&self) -> Option<&[u8]> {
        (self.tx_hash.len() == HASH_LEN).then_some(self.tx_hash.as_slice())
    }

    /// Returns the auth digest in display byte order, or `None` if the field
    /// is absent or not 32 bytes.
    ///
    /// Only v5+ transactions have an auth digest; Zebra sends an empty field
    /// for older ones.
    pub fn auth_digest_display_order(&self) -> Option<&[u8]> {
        (self.auth_digest.len() == HASH_LEN).then_some(self.auth_digest.as_slice())
    }
}

impl BlockHashAndHeight {
    /// Returns the block hash in display byte order, or `None` if the field
    /// is not 32 bytes.
    pub fn hash_display_order(&self) -> Option<&[u8]> {
        (self.hash.len() == HASH_LEN).then_some(self.hash.as_slice())
    }
}

/// The maximum number of chain tip hashes a [`NonFinalizedStateChangeRequest`]
/// may carry.
///
/// Zebra's non-finalized state holds at most this many chains
/// (`MAX_NON_FINALIZED_CHAIN_FORKS` in `zebra-state`), so a caller can never
/// legitimately have more known tips than this. The server rejects larger
/// requests with `invalid_argument` — bounding untrusted input up front
/// rather than allocating a set sized by the request — so build requests
/// against this limit rather than discovering it as a stream error.
pub const MAX_CHAIN_TIP_HASHES: usize = 10;

impl NonFinalizedStateChangeRequest {
    /// Returns this request's chain tip hashes as 32-byte display-order
    /// slices, or `None` if any hash is not exactly 32 bytes.
    ///
    /// Wrong-length hashes are rejected by the server with `invalid_argument`;
    /// this surfaces the same check up front. The number of hashes is checked
    /// separately against [`MAX_CHAIN_TIP_HASHES`].
    pub fn chain_tip_hashes_display_order(&self) -> Option<impl Iterator<Item = &[u8]> + '_> {
        let all_valid = self
            .chain_tip_hashes
            .iter()
            .all(|hash| hash.len() == HASH_LEN);

        all_valid.then(|| self.chain_tip_hashes.iter().map(|hash| hash.as_slice()))
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
        assert_eq!(msg.tx_hash_display_order(), Some(&msg.tx_hash[..]));
        msg.auth_digest = vec![1; 32];
        assert_eq!(msg.auth_digest_display_order(), Some(&msg.auth_digest[..]));

        let tip = BlockHashAndHeight {
            hash: vec![2; 32],
            height: 1,
        };
        assert_eq!(tip.hash_display_order(), Some(&tip.hash[..]));
        assert_eq!(BlockHashAndHeight::default().hash_display_order(), None);
    }

    #[test]
    fn chain_tip_hash_accessors_match_server_validation() {
        let hash = || vec![3u8; 32];

        let req = NonFinalizedStateChangeRequest {
            chain_tip_hashes: vec![hash(), hash()],
        };
        let tips: Vec<&[u8]> = req
            .chain_tip_hashes_display_order()
            .expect("32-byte hashes are valid")
            .collect();
        assert_eq!(tips.len(), 2);
        assert!(tips.iter().all(|t| t.len() == 32));

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

        // More hashes than Zebra has non-finalized chains is never legitimate.
        let req = NonFinalizedStateChangeRequest {
            chain_tip_hashes: (0..=MAX_CHAIN_TIP_HASHES).map(|_| hash()).collect(),
        };
        assert!(req.chain_tip_hashes.len() > MAX_CHAIN_TIP_HASHES);
    }
}
