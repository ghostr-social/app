# Handoff — Rust Nostr engine cutover

Goal: **Rust owns every Nostr relay read and write.** Dart keeps Flutter UI,
settings, and local private-key operations. Keys never cross FFI.

## Current production boundary

Rust owns:

- the only relay client and socket pool;
- generic single and batched Nostr filters;
- feed, search, profile, reaction, comment, activity, and social-list reads;
- NIP-50 search and NIP-65 outbox routing;
- signed-event validation, relay selection, and broadcast;
- the bounded warm event cache and offline fallback;
- relay/data-usage/storage configuration, including live updates;
- account-session reset of feeds, queued work, cache, profiles, social graph,
  outbox state, and dynamically discovered relays;
- media admission, scheduling, retrieval, cache, and loopback playback.

Dart owns:

- UI and application orchestration;
- local account activation and event signing;
- local NIP-51 decryption/model conversion;
- settings persistence and typed FFI mapping;
- Blossom HTTP upload.

`buildNdk()` has an empty bootstrap relay list. Production NDK usage is limited
to local accounts/signers, parsing/decryption helpers, local memory cache, and
Blossom HTTP. `ProductionNostrServices` does not expose its NDK instance or a
broadcast escape hatch.

## Live implementation map

- `lib/app/production_nostr_services.dart` composes the Nostr boundary.
- `lib/platform/nostr/rust_nostr_event_client.dart` maps feature reads and
  locally signed writes onto the Rust event API.
- `lib/platform/nostr/rust_broadcast_adapter.dart` is the sole signed-event
  serialization edge.
- `lib/platform/nostr/rust_nostr_session.dart` coordinates account resets.
- `lib/features/video_catalog/data/rust_feed_remote_source.dart` adapts Rust
  feed snapshots to the app's pull-shaped catalog contract.
- `lib/platform/media/rust_engine_configuration_mapper.dart` is the sole
  settings-to-engine configuration mapper.

## Important invariants

- All feature reads use `NostrEventClient`, whose production implementation is
  `RustNostrEventClient`.
- Dart signs canonical NIP-01 events; `ffi_broadcast_event` verifies and sends
  the validated `SignedNostrEventJson` through Rust.
- Engine startup receives typed read relays, search relays, data usage, and the
  full storage budget. A native start permit makes the installed-engine check
  and installation atomic. Generated enums carry data usage, feed kind, and
  media delivery across FFI.
- Saving settings compensates a rejected engine update by restoring prior
  persistence. If that compensation itself fails, the save surfaces an
  explicit persisted/live divergence failure and directs a restart; it never
  reports the initiating error as if rollback had succeeded.
- Sign-in and sign-out reset Rust before changing the local signer. If the
  local signer transition fails, the previous native account is restored. A
  failed compensation surfaces an explicit local/native divergence failure;
  Rust account guards reject mismatched work until restart/reconciliation.
- A feed request captures its account and native session generation together.
  Rust validates both while holding the relay-reset read barrier, before any
  feed state is allocated or mutated. This rejects late work after account
  switches, sign-out, and same-account resets.
- Relay replacement and session reset reconcile the SDK client's actual pool,
  so removed, dynamically discovered, or prior-account relays cannot receive
  later fallback reads or writes.
- `test/architecture/nostr_transport_ownership_test.dart` guards the ownership
  boundary and the small allowlist of local NDK adapters.

## Verification

The final command results and post-cutover Android smoke pass are recorded in
the task's completion report and
[MANUAL_VERIFICATION.md](./MANUAL_VERIFICATION.md).

No known production Dart relay transport remains.
