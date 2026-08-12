# Progressive Video Debug Web App

Start the standalone Rust service from the repository root:

```sh
make web
```

Every invocation first stops the previous dashboard owned by this branch.
That clears its in-memory event and video registries. It then deletes shared
and abandoned `rust/target/video-debug-cache*` directories, uses a new empty
private cache, and removes that cache on exit. The lifecycle owner record lives
under `rust/target/video-debug-web`; cleanup signals only the launcher and
debug-binary PIDs recorded there after their command paths match this branch.
An abandoned owner file whose PID was reused is discarded without signaling
that unrelated process.

The new process immediately queries the relays again. Seeing the same event IDs
after restart can therefore be legitimate fresh relay discovery; it is not
evidence that the previous process or its event registry survived.

It prints the exact page URL, for example:

```text
Video debug dashboard: http://127.0.0.1:43595/debug
```

Open that URL in a browser. The command starts the Rust delivery engine
and the Rust Nostr discovery runtime directly; it does not build or start Dart
or Flutter. It opens the signed-out main video feed against the configured
relays, then sends discovered videos into the shared feed projection.
Progressive videos enter the real prefetch and partial-storage pipeline; HLS
videos stay outside that cache and play through Rust-owned HLS sessions. The
Add video dialog remains available for testing a progressive HTTP(S) URL.

The page, JavaScript, CSS, and pinned hls.js player are compiled into the Rust
debug binary, so no separate web build, asset server, or runtime CDN is
required. Safari uses native HLS; other supported desktop browsers use the
embedded player against the same Rust manifest and asset gateway. The assets
are behind the opt-in `video-debug-web` Cargo feature and a non-mobile
debug-build guard.

The debugger is an adapter over the same staged pipeline and feed projection
used by mobile. See [VIDEO_PIPELINE.md](./VIDEO_PIPELINE.md). It does not own a
parallel candidate or download registry.

The dashboard uses three desktop columns: a portrait device playback preview,
an ordered Playing / Up next / Played queue, and a delivery inspector. Video
input and live network simulation controls are kept in modal dialogs.

## What it shows

- every progressive video in the current adaptive frontier and every HLS feed row
- live Nostr feed stage, relay connection status, and discovered-video count
- Nostr event, creator, and title metadata attached to each discovered video
- source media host and mirror count
- stored and total bytes, percentage, and exact stored ranges
- video duration and the duration-equivalent currently buffered
- observed download rate and estimated remaining time
- complete, partial, queued, and actively-downloading states
- current partial-store usage and active connections by media host
- recent adaptive plans with exact ranges, playable gain, utility, authority,
  commitment, reason, retained work, eviction evidence, and discovery demand
- playback through the existing `/video.mp4` progressive route
- HLS playback through acquired `/hls/{session}/...` gateway routes
- direct registration of progressive media without Dart or Flutter

## Network simulation

The controls update the running downloader without restarting it:

- one aggregate bandwidth budget in Kbps, shared fairly by active media transfers
- latency in milliseconds before each range request
- maximum simultaneous range connections per media host
- deterministic packet-loss injection in basis points

Zero disables the corresponding limit. Limits affect progressive media range
downloads. They do not alter Nostr discovery, HLS proxying, or the loopback
telemetry requests made by the debug page.

## Automated delivery acceptance

`make video-user-e2e-impairments` runs the browser acceptance rows as separate
processes with separate private caches. Its `adaptive_plans` row holds focus on
video zero and requires a healthy policy-selected frontier to expand, without
asserting a fixed number of videos or bytes per candidate. Every row retains a
bounded history of production `AllocationPlan` evidence and exact origin
chronology. Moving impairment rows prove distinct outcomes: network conditions
change allocation evidence without restarting paid ranges, rapid navigation
adds explicit coverage, storage pressure emits exact evictions, and a failed
source reallocates the same range to a viable mirror. QoE gates independently
cover transition latency, rebuffer, cancellation waste, speculative-byte
ceiling, and zero duplicate completed origin bytes.

The exact release budgets and evidence rules are documented in
[VIDEO_QOE_TARGETS.md](../standards/VIDEO_QOE_TARGETS.md).

## HTTP surface

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/debug` | Debug screen |
| `GET` | `/debug/api/state` | Current storage, connection, and video snapshot |
| `PUT` | `/debug/api/focus` | Move native delivery focus to a discovered Nostr video |
| `POST` | `/debug/api/hls` | Acquire a Rust HLS playback session for a feed row |
| `DELETE` | `/debug/api/hls/{session}` | Release a Rust HLS playback session |
| `POST` | `/debug/api/videos` | Register a progressive video with the Rust delivery engine |
| `PUT` | `/debug/api/network` | Replace the live network profile |
| `PUT` | `/debug/api/storage` | Replace the partial-store byte budget |

The network profile JSON has this shape:

```json
{
  "bandwidth_kbps": 768,
  "latency_ms": 350,
  "max_connections_per_host": 1,
  "packet_loss_bps": 0
}
```

The embedded server remains bound to loopback, so this development control
surface is not exposed to other devices on the network.

## Nostr relays

The standalone debugger defaults to the same public read and search relay sets
used by the application defaults. Override either comma-separated list without
involving Dart:

```sh
GHOSTR_NOSTR_RELAYS=wss://relay.example \
GHOSTR_NOSTR_SEARCH_RELAYS=wss://search.example \
make web
```
