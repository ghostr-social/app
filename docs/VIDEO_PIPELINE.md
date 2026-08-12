# Video Pipeline

Release-level latency, rebuffering, cancellation, and prefetch budgets are
defined in [VIDEO_QOE_TARGETS.md](../standards/VIDEO_QOE_TARGETS.md).

The progressive-video path is one staged pipeline:

```text
relay tasks
  │ raw events (bounded backpressure; no event-count truncation)
  ▼
CandidateRegistry
  │ validated, parsed, coordinate-deduplicated candidates
  ▼
MetadataProbePool
  │ validated size, range support, source health, and media layout
  ▼
AdaptivePlayabilityPolicy
  │ exact admitted ranges, utility, authority, commitment, and reason
  ▼
MutablePriorityQueue
  │ replaceable policy-selected range work
  ▼
DownloadWorkers
  │ bounded, cancellable transfers
  ▼
CacheRegistry
  │ ready / partial / complete progressive entries
  ▼
FeedProjection
  ├─ mobile FFI snapshots
  └─ debug web adapter
```

## Ownership

- Relay tasks own transport only. Main-feed retrieval is continuous: filters
  within a page run concurrently, history pages advance without UI commands,
  and a bounded history burst returns to the newest head before continuing.
  Relay ingestion never waits for metadata probing, downloading, caching, or
  feed projection.
- Relay tasks forward every raw event while the retrieval remains open.
- `CandidateRegistry` is the only raw-event admission boundary. It validates
  and parses an event once, rejects non-video events, deduplicates exact
  repeats, and replaces older revisions of the same addressable coordinate.
- Candidate admission reaches delivery immediately. A mobile or web focus
  round trip is not required to start metadata probing or initial prefetch.
- `MetadataProbePool` bounds concurrent probes. It learns byte length, range
  support, and enough head/tail container evidence to expose playable ranges.
- `AdaptivePlayabilityPolicy` is pure. Each snapshot combines playback
  buffer, navigation probabilities and swipe rate, per-video bitrate and
  timeline, stored and in-flight ranges, throughput/RTT/loss/connection
  evidence, origin health, and storage pressure. Its `AllocationPlan` is the
  sole origin-admission authority.
- `MutablePriorityQueue` is replaced after focus, demand, probe, retry, cache,
  or configuration changes. Leaving the plan cancels active work; entering or
  moving within it changes the next worker grant.
- `DownloadWorkers` own active range tasks and their cancellation handles.
- `CacheRegistry` is the gateway allow-list and the delivery-state projection.
  It retains every live admitted candidate, not only the current focus window.
- `FeedProjection` is built once from canonical feed state. Mobile and the
  debug web app are adapters over that same projection.

## Priority contract

Before a UI supplies focus, projected candidates can begin bounded metadata
probing. Once focus is supplied, the policy first protects the current
playback buffer, then buys playable time for likely transitions, preserves
useful committed work, and expands speculative coverage only when measured
network, connection, and storage capacity permit it. The frontier is not a
fixed prefix: it may contain one candidate or many, and both its breadth and
per-candidate depth emerge from the current snapshot.

Every origin body request must exactly match an allocation recorded before
the request began. Replanning subtracts stored and live in-flight ranges and
keeps useful commitments, so completed origin ranges never overlap. A
range-blind file is represented as one complete-file opportunity and is
deferred until existing playable coverage can pay its delivery time; a bounded
tail layout probe may run earlier to determine whether that deferral is needed.
All validated candidates remain in the cache registry; focus changes value and
scheduling eligibility, not identity.

HLS media shares relay admission and feed projection but stays outside the
progressive range-download queue; it continues through the HLS session gateway.

The browser acceptance runs every row with a fresh cache. A healthy held-focus
row proves that policy-selected coverage expands without asserting a candidate
count. Moving-focus rows retain exact plan history and origin chronology, then
verify network replanning without paid-byte restarts, rapid-navigation
coverage, exact storage-pressure eviction, and source reallocation alongside
latency, rebuffer, cancellation, and duplicate-byte budgets.
