# Maintenance helpers for zebra-indexer-proto
#
# The goal is the best possible "cargo build" experience for downstream users.
# Generated code is committed. Normal builds require no protoc.

# The proto source lives in this repo (vendored from Zebra).
# To update it:
#   1. Copy the latest indexer.proto from a Zebra release or git tree:
#        cp /path/to/zebra/zebra-rpc/proto/indexer.proto proto/indexer.proto
#   2. Run regeneration (requires protoc):
#        make regenerate
#   3. Review the diff in proto/__generated__/ and commit.

.PHONY: regenerate check clean

# Regenerate committed Rust bindings + descriptor from the .proto.
# Requires protoc to be installed.
regenerate:
	cargo build --features regenerate

# Quick sanity check that default build (the important path) works.
check:
	cargo check
	cargo test

clean:
	cargo clean
