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

pub mod zebra {
    pub mod indexer {
        pub mod rpc {
            include!("../proto/__generated__/zebra.indexer.rpc.rs");
        }
    }
}

pub use zebra::indexer::rpc::*;

pub use tonic::transport::Channel;

pub type ZebraClient =
    zebra::indexer::rpc::indexer_client::IndexerClient<Channel>;

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
}
