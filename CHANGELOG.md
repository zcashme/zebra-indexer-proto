# Changelog

Notable changes to `zebra-indexer-proto`. The crate tracks Zebra's indexer
gRPC interface, not Zebra's own version numbers.

## 2.4.0 (2026-06-27)

- Dropped the `Client` back-compat alias.
- Trimmed explicit client/server re-exports; the glob re-export plus
  `ZebraClient`, `Channel`, and `ConnectError` remain. (This trim landed
  minutes after the version bump and shipped as part of the same 2.4.0
  publish. Tagged retroactively as the tree actually published.)
- proto: byte-identical to `ZcashFoundation/zebra` (which vendored our file
  into `zebra-rpc/proto/indexer.proto` for v6.3.0).

## 2.3.0 (2026-06-27)

- Added the `ZebraClient` type alias (`IndexerClient<Channel>`), so callers
  don't need `tonic` in their own manifests to name the client.
- README: serde usage docs.

## 2.2.0 (2026-06-27)

- All generated message types now derive `serde::Serialize` / `Deserialize`,
  so payloads can be re-emitted as JSON or cached without hand-rolled encoders.

## 2.1.0 (2026-06-27)

- Added `Channel` and `ConnectError` re-exports for the same reason as
  `ZebraClient`.
- Replaced the Makefile with an optional `regenerate` feature
  (`cargo build --features regenerate`).

## 2.0.1 (2026-06-27)

- Reformatted `indexer.proto` to match Zebra's in-tree copy exactly
  (semicolons after messages, comment placement), making byte-level
  diffing against upstream possible.

## 2.0.0 (2026-06-27)

- The "it just builds" release: generated code is now committed under
  `proto/__generated__/`, so downstream builds need no `protoc` and no
  codegen dependencies. Added the file descriptor set for reflection.

## 0.1.0 – 0.1.3 (2026-06-26)

- Initial sketch: vendored proto, build-time generation, early re-export
  shape. Superseded by 2.0.0.
