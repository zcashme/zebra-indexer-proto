//! Rust bindings for Zebra's indexer gRPC interface.
//!
//! Generated from `indexer.proto` in the Zebra repository.
//! This crate provides the gRPC client and server types for Zebra's
//! Indexer service, with zero Zebra dependencies.

tonic::include_proto!("zebra.indexer.rpc");

// Re-export the client for convenience
pub use indexer_client::IndexerClient;