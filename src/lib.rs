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

/// The raw generated module tree (package = zebra.indexer.rpc).
pub mod zebra {
    pub mod indexer {
        pub mod rpc {
            include!("../proto/__generated__/zebra.indexer.rpc.rs");
        }
    }
}

// Re-export all generated items (messages, enums, etc.) at the crate root
// for the most convenient usage (matching `tonic::include_proto!` ergonomics).
pub use zebra::indexer::rpc::*;

// First-class, explicit re-exports for the gRPC service.
pub use zebra::indexer::rpc::indexer_client::IndexerClient;
pub use zebra::indexer::rpc::indexer_server::{Indexer, IndexerServer};

// Also expose the server module directly (like Zebra does internally).
pub use zebra::indexer::rpc::indexer_server;

/// The transport channel used by the default client. Re-exported so callers
/// can name [`ZebraClient`] without adding `tonic` to their own manifests.
pub use tonic::transport::Channel;

/// A ready-to-use client type with the default transport pinned, so callers
/// never have to spell `IndexerClient<Channel>`.
///
/// ```no_run
/// # async {
/// let mut client = zebra_indexer_proto::ZebraClient::connect("http://127.0.0.1:8230").await?;
/// # let _: zebra_indexer_proto::ZebraClient = client;
/// # Ok::<(), zebra_indexer_proto::ConnectError>(()) };
/// ```
pub type ZebraClient = IndexerClient<Channel>;

/// Back-compat alias for [`ZebraClient`] (kept since 2.1.0; prefer `ZebraClient`).
pub type Client = IndexerClient<Channel>;

/// The error returned by [`ZebraClient::connect`] (an alias for `tonic::transport::Error`).
/// Re-exported so callers using `?` on a connect call don't need `tonic` in their manifest.
pub use tonic::transport::Error as ConnectError;

/// Encoded protobuf file descriptor set for this service.
/// ```
/// let _descriptor: &[u8] = zebra_indexer_proto::FILE_DESCRIPTOR_SET;
/// ```
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
        // Just ensure the symbols are public and have the expected names.
        fn _assert_client(_: IndexerClient<tonic::transport::Channel>) {}

        // These are the main types users need to implement or construct servers.
        let _ = std::any::type_name::<IndexerServer<()>>();
        // The trait name is available for `impl Indexer for MyType`.
        let _name = std::any::type_name::<fn() -> IndexerServer<()>>();
    }

    #[test]
    fn descriptor_is_non_empty() {
        assert!(!FILE_DESCRIPTOR_SET.is_empty());
        // Very rough sanity: protobuf descriptors often start with a length or specific bytes.
        assert!(FILE_DESCRIPTOR_SET.len() > 100);
    }
}
