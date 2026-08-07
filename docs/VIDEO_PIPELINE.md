# Video Pipeline

The progressive-video path is one staged pipeline:

```text
relay tasks
  │ raw events (bounded backpressure; no event-count truncation)
  ▼
CandidateRegistry
  │ validated, parsed, coordinate-deduplicated candidates
  ▼
MetadataProbePool
  │ candidates with a usable byte length and a live source
  ▼
MutablePriorityQueue
  │ replaceable ordered range work
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
- `MetadataProbePool` bounds concurrent probes. Candidates without a usable
  byte length cannot enter the download queue.
- `MutablePriorityQueue` is replaced after focus, demand, probe, retry, cache,
  or configuration changes. Leaving the plan cancels active work; entering or
  moving within it changes the next worker grant.
- `DownloadWorkers` own active range tasks and their cancellation handles.
- `CacheRegistry` is the gateway allow-list and the delivery-state projection.
  It retains every live admitted candidate, not only the current focus window.
- `FeedProjection` is built once from canonical feed state. Mobile and the
  debug web app are adapters over that same projection.

## Priority contract

Before a UI supplies focus, newly admitted candidates are ranked newest first
and can begin prefetch immediately. Once focus is supplied, the focused window
controls download ordering. All admitted candidates remain eligible for the
probe pool and cache registry, so focus changes priority rather than identity
or admission.

HLS media shares relay admission and feed projection but stays outside the
progressive range-download queue; it continues through the HLS session gateway.
