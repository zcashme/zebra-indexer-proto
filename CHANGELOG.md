# Changelog

Notable changes to `zebra-indexer-proto`. The crate tracks Zebra's indexer
gRPC interface, not Zebra's own version numbers.

## 3.0.0 (2026-11-09)

Checked against Zebra v7.0.0.

- Breaking: `tx_hash_display_order`, `auth_digest_display_order`,
  `BlockHashAndHeight::hash_display_order` and
  `chain_tip_hashes_display_order` return `&[u8; 32]` instead of `&[u8]`.
- Breaking: `MAX_CHAIN_TIP_HASHES` is 100, the limit Zebra enforces on every
  network (`MAX_NON_FINALIZED_CHAIN_FORKS_TEST_NETWORKS`). It was 10.
- Added `HashOrHeight`, with `From<HashOrHeight> for BlockRequest` and
  `BlockRequest::hash_or_height()`.
- Added `BlockAndHash::hash_display_order()`.
- Added constructors that encode each field rule:
  `BlockHashAndHeight::new`, `BlockAndHash::new`,
  `NonFinalizedStateChangeRequest::new` (`None` above the limit) and
  `MempoolChangeMessage::new`.
- The "Protocol behavior notes" are "Zebra server facts", each citing
  `zebra-rpc/src/indexer/methods.rs` at v7.0.0. Corrected: `chain_tip_change`
  sends the current tip at once once Zebra's tip has changed, and a slow
  consumer's stream ends cleanly with no status.
- proto: byte-identical to Zebra v7.0.0 again. The two `tx_hash` and
  `auth_digest` comment lines are Zebra's; the display-order fact moved to
  the accessor docs. Regenerated bindings carry `#[allow(clippy::all)]` from
  the current `prost-build`.

## 2.5.0 (2026-06-27)

- Added the typed mempool layer:
  - `MempoolChangeKind` — the `ChangeType` enumeration prost generates from
    the proto, re-exported under the name Zebra's own stack uses for the
    same taxonomy. The 0 = Added / 1 = Invalidated / 2 = Mined correspondence
    is now owned by code generation instead of by comments and hand-written
    matches downstream.
  - `MempoolChangeMessage::kind()` — decodes `change_type` to
    `Option<MempoolChangeKind>`, `None` on unknown wire values (proto3 enums
    are open).
  - `MempoolChangeMessage::tx_hash_display_order()` and
    `auth_digest_display_order()` — `Some` iff the field is exactly 32 bytes.
  - `BlockHashAndHeight::hash_display_order()` — the same 32-byte discipline
    for tip hashes.
  - `NonFinalizedStateChangeRequest::chain_tip_hashes_display_order()` —
    `Some` iff every hash is exactly 32 bytes, mirroring the server's up-front
    validation.
  - `MAX_CHAIN_TIP_HASHES` — the server's limit on chain tip hashes per
    resume request (Zebra's `MAX_NON_FINALIZED_CHAIN_FORKS`); larger requests
    are rejected with `invalid_argument`.
- Added a "Protocol behavior notes" section to the crate docs: no initial
  message on `chain_tip_change`, resume-request limits, and stream
  termination semantics (drop on close or slow consumption — resubscribe
  rather than treat as fatal). These are Zebra implementation choices, noted
  as such.
- CI: install protoc (ubuntu runners don't ship it; `--all-features` enables
  `regenerate`, which needs it for both tests and the bindings check).
- proto: documented `tx_hash` and `auth_digest` as display order (matching
  every other hash in the file and Zebra's server, which sends
  `mined_id().bytes_in_display_order()`), and noted `auth_digest` is empty
  for transactions without one. The regenerated bindings carry the comments.
  This is a deliberate divergence from Zebra's in-tree copy until it syncs
  the two comment lines — the daily CI sync check fails in the interim.

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
