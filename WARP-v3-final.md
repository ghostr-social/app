# WARP v3: Watch-Aware, Origin-Adaptive Retrieval Planning

## Final consolidated design and implementation specification for heterogeneous Nostr video feeds

**Revision:** 3.0 — 4 September 2026  
**Document status:** Implementation baseline; performance hypotheses remain unmeasured.  
**Supersedes:** WARP v2 specification and its companion review. This document is self-contained; the earlier documents are historical, not additional requirements.  
**Design lineage:** The supplied August 2026 manuscript; the supplied WARP v2 corrections; and the subsequently agreed extensions for virtual packaging, reusable/authenticated readiness maps, startup-oriented publication, staged commitment, and cumulative-deficit buffering.

> **Central rule:** Prepare the smallest dependency-complete state that enables useful playback; retain reusable knowledge and verified progress; acquire additional payload no earlier than necessary to protect the intended playback under the chosen service assumptions and resource limits.

“MUST”, “MUST NOT”, “SHOULD”, and “MAY” specify WARP implementation requirements. They do not impose requirements on ordinary Nostr publishers or third-party origins. Normative algorithms state their assumptions; numerical starting values are experiment settings, not optimal constants. A build implements the core plus explicitly declared extensions. “Final” denotes this consolidated implementation baseline, not a claim of production validation or permanent protocol standardization.

## Abstract

WARP is a retrieval and playback controller for a short-video client whose media may be hosted on unrelated servers, use incompatible containers, expose incomplete metadata, or lack reliable byte-range access. It represents playback as a graph of concrete dependencies, separates byte availability from decoder and presentation readiness, and controls all media acquisition through one budgeted, generation-aware broker. A deterministic scheduler protects active playback and the intended next item before spending resources on speculation. Optional extensions reuse compiled media structure, authenticate useful pieces before full-object completion, and expose virtual media packaging without materializing a second complete payload. A cooperative publishing profile makes startup dependencies small while preserving a real continuation path. Latest-safe scheduling and finite-horizon service-deficit analysis reduce premature acquisition without treating predicted Internet service as guaranteed. The specification defines interfaces, trust transitions, failure behavior, a versioned sidecar format, rollout gates, and acceptance scenarios. It makes no measured performance claim.

## Executive design

WARP is a delivery controller, not a recommender. Ordinary URLs remain playable through bounded discovery and compatible adapters. Optional WARP sidecars are application extensions, not an existing Nostr standard and not a prerequisite for using the client.

One transfer broker owns external requests, accounting, deduplication, and encoded-byte storage. Container adapters compile actual media structure into dependency graphs. A playback adapter owns demuxing, decoding, presentation, and accurate buffer telemetry. The scheduler operates on these shared contracts instead of assuming that downloaded bytes are immediately visible frames.

There are three kinds of reusable work: **knowledge** (validated structure and indexes), **payload** (generation-consistent or authenticated bytes), and **activation** (a prepared playback backend). Their costs, validity, and eviction rules differ. Preparing cheap knowledge early is not permission to fetch an entire video early.

The core uses bounded discovery, current/next protection, finite resource leases, local compiled-index reuse, and a deterministic scheduler. The design includes the following separately gated extensions:

| Profile | Meaning | Required cooperation |
|---|---|---|
| `CORE/3` | Safe heterogeneous retrieval, explicit readiness, local indexes, conservative continuation planning | None beyond a usable ordinary media endpoint |
| `WRM/1` | Content-bound readiness sidecars and optional authenticated block lists | A publisher commitment or a locally established binding for authentication; untrusted maps are hints only |
| `VPK/1` | Locally compiled virtual fragmented-MP4 packaging | No publisher change; a supported codec/layout and accessible source dependencies |
| `PUB/1` | Small startup dependencies, full baseline rendition, optional aligned upgrades and replicated startup resources | A cooperating uploader/publisher |
| `JIT/1` | Latest-safe payload commitment using a valid joint reservation calendar | Sufficient service evidence or explicitly controlled service assumptions |
| `LOOKAHEAD/1` | Contingent digital-twin proposals, advanced risk estimates, optional adaptive prices | No publisher change; measured benefit over the deterministic core |

A profile being specified does not mean it is enabled. A release manifest MUST name enabled profiles, exact dependency/protocol snapshots, supported backend/device combinations, numerical caps, integrity policy, and passed acceptance cases. Unsupported extensions fail closed to the ordinary path; they do not make an otherwise usable post disappear.

Three distinctions remain fundamental:

- **Bytes versus playback:** fetched ranges, complete startup dependencies, prepared decoders, and visible presentation are different states.
- **Evidence versus verification:** a declaration or successful decode does not authenticate partial bytes. Cryptographic state is discrete, never a probability.
- **Goals versus guarantees:** low interruption probability is an observed service objective. A reservation calendar is conditional on its service assumptions; resource leases enforce only their stated accounting boundary.

## Reading and implementation order

Sections 1–4 establish scope, invariants, and ownership. Section 5 specifies discovery, HTTP correctness, and readiness-map trust. Section 6 defines usable media plans, virtual packaging, and publication. Sections 7–9 define predictions, buffer feasibility, budgets, and scheduling. Sections 10–13 cover storage, interfaces, tests, and security. Section 14 gives release stages. Appendix D fixes the `WRM/1` wire contract; Appendix E supplies pure-function reference contracts and test scope. The accompanying implementation pack contains the matching JSON Schema, synthetic vectors, reference functions, and tests—not a complete player or Rust service.

# 1. Scope, goals, and non-goals

## 1.1 Reference deployment

The reference deployment is a **single-user native application or installed companion with a Rust retrieval service**, local encoded-byte cache, and a native/WebView/browser playback adapter. A loopback HTTP gateway is one delivery adapter, not an assumption that any public web page can fetch arbitrary origins.

A browser-only deployment is a separate profile. Its controlled fetch path requires origin cooperation with browser fetch policies, or an explicitly operated remote gateway. A service worker does not remove Fetch/CORS restrictions. Public-page access to a local service also requires a tested browser permission and security integration; do not assume localhost is unrestricted. [S8, S17]

For a remote gateway, model origin-to-gateway and gateway-to-device paths separately. Origin ingress, gateway egress, and device data allowance are different budgets. This version's local cache and privacy assumptions must not silently become shared multi-user assumptions.

## 1.2 Priority order

The controller applies these priorities lexicographically:

1. Enforce security, byte-generation consistency, authorization, and explicit resource limits.
2. Protect the active presentation and fulfill explicit user intent, including seeks and backward navigation.
3. Prepare the intended next item and an allowed reserve of future starts with sufficient continuation coverage.
4. Reduce transferred bytes, request amplification, storage, and energy while meeting the playback target.
5. Improve quality and reuse only from remaining capacity.

A speculative future video MUST NOT consume a resource reservation needed by a feasible current-playback rescue. “Feasible” includes completion before the actual playback deadline, not just a free worker slot.

## 1.3 Scope of initial media support

Each build publishes a tested capability matrix by playback backend, container, codec configuration, and delivery mode. It MUST distinguish unsupported-by-profile from malformed or unsafe.

The initial target is clear, finite video-on-demand: native progressive MP4; fragmented MP4 where the backend accepts it; supported native WebM; and an HLS VOD adapter. DASH and other containers may be enabled through tested adapters rather than silently claimed as universal support. Codec support is configuration-specific: container MIME alone is insufficient.

Live/low-latency streaming, DRM, arbitrary encrypted streams, multi-tenant transcoding, end-to-end reinforcement learning, and authenticated tree/proof protocols beyond the defined `WRM/1` block list are outside this implementation baseline. Encryption is not inherently unsafe; unsupported encryption is a capability outcome. `WRM/1`, `VPK/1`, and `PUB/1` are optional additions to the core, never mandatory properties of Nostr media.

# 2. Object model and playback readiness

## 2.1 Separate identities

| Entity | Meaning and identity |
|---|---|
| **Feed item** | An ordered recommendation or explicitly selected post; identifies semantic intent independently of delivery. |
| **Event revision** | The exact signed Nostr event used for a decision. Addressable updates produce new revisions. |
| **Presentation** | The selected video, audio, timeline, captions, and playback backend. Separate audio belongs here, not in an unrelated video candidate. |
| **Variant** | A distinct encoding/container or manifest-based presentation option. Related variants are not automatically switch-compatible. |
| **Endpoint** | A URL plus request/authorization context. A fallback is an endpoint claim, not proof of identical bytes. |
| **Representation generation** | One byte sequence under one origin/resource/request context and version evidence. Sparse intervals never cross generations implicitly. |
| **Verified local object** | Complete local bytes with a computed digest. Matching an expected digest additionally binds them to that declaration. |
| **Readiness unit** | A dependency-complete playable interval or other operational milestone, with explicit required resources and tracks. |
| **Compiled index** | Locally interpreted source structure, bound to content/generation, parser version, selected tracks, and access context. |
| **Readiness map** | A bounded sidecar that proposes resources and playback dependencies; binding/authenticity and structural validation are independent. |
| **Authenticated resource view** | A map/resource-bound view assembled only from verified authentication units; distinct from origin-scoped provisional generations. |
| **Virtual representation** | An immutable locally compiled output plan with generated headers and fixed source slices; its identity is not the original file hash. |
| **Reservation calendar** | Expiring joint dispatch/resource plan for protected dependency work, conditional on an explicit service envelope. |

`x` names an expected file digest; `ox` names the pre-transformation file according to NIP-94. They do not establish interchangeable encodings, aligned timelines, or authenticated chunks. [S3]

## 2.2 Readiness states

Maintain independent **byte**, **decoder**, and **verification** state. Do not encode all three in `downloaded: bool` or `confidence: f32`.

| Playback state | Required evidence | What it does not promise |
|---|---|---|
| `Discovered` | An admissible media reference | Container, length, compatibility, or availability |
| `MetadataParsed` | Bounded structural analysis and track selection | A decodable frame |
| `StartupBytesReady` | Initialization, random-access dependencies, and required video/audio samples locally available | Decoder allocation or immediate presentation |
| `DecoderReady` | The chosen backend has prepared this exact presentation and can activate it; include an observed first-frame preparation result where supported | A frame already visible to the user |
| `Presented` | Presentation telemetry for this playback epoch | Sufficient future buffer |
| `BufferedUntil(t)` | Contiguous playable coverage to presentation time `t`, for all required tracks | Playability beyond `t`, or preservation after eviction |

A readiness certificate contains `(presentation_id, object_generations, backend_epoch, capability_epoch, track_set, interval, dependencies, leases, evidence_level)`. Eviction, a seek, a track change, a decoder reset, or a generation change invalidates the affected certificate.

A warm decoder is scarce. The reserve may contain several `StartupBytesReady` assets but only one prewarmed next player. Cold reserve items incur measured decoder-start delay in planning. A poster or BlurHash never counts as a first frame or a ready video.

## 2.3 Useful coverage

For presentation time `p`, required track set `A`, and selected playback rate `r > 0`, define:

```text
B_media = length of the contiguous intersection of buffered track intervals starting at p
B_wall  = B_media / r
```

Track times MUST be mapped to the presentation timeline before intersection. Do not use the furthest buffered timestamp when there is a gap. During a pause, playback does not consume this coverage; speculative fetching still obeys policy.

A startup target is **first frame plus a bounded continuation cushion**, not first frame alone. Use the cumulative service-deficit test in Section 7.5 over a meaningful near-term horizon, including authentication, packaging, required audio, and decoder/dispatch margins. Fixed minimum cushions are bootstrap fallbacks, not proofs of safety. When the required coverage exceeds watch-aware resource limits or has no feasible continuation plan, record degraded readiness rather than silently clipping the requirement and declaring the presentation stall-safe.

## 2.4 Retrieval DAG

Each readiness unit lists its transitive dependencies: manifest snapshots, initialization data, selected audio, codec configuration, random-access samples, media ranges or segments, and any necessary conversion. Shared dependencies are represented once. Dependencies are typed as retained bytes, validated structure, or live backend state. A still-valid decoder/demux state token can satisfy an already consumed predecessor without re-fetching all earlier encoded samples. The static `WRM/1` span closure is a cold-acquisition description; the runtime subtracts dependencies satisfied by valid cache or backend-epoch tokens. Seeking, resetting, or changing tracks invalidates the relevant state token and requires a valid random-access closure again. Byte eviction does not erase already presented history or necessarily evict decoded buffers; only certificates actually depending on those bytes are invalidated.

An action is bounded work toward one milestone: discover bytes, fetch an interval, extend an existing readable response, prepare a decoder, or perform an admitted transform. A failed probe does not prove an object is full-only; it may mean **unknown under the discovery budget**. Keep unknown, transient failure, unsupported, and malformed states separate.

## 2.5 Assurance at the playback boundary

Each readiness certificate carries the actual verification level of its dependency closure. The default `GenerationConsistent` policy permits provisional playback under the ordinary generation rules. Optional `AuthenticatedBeforeUse` requires complete-object or authorized resource/block verification before bytes are released for the required playback scope. A publisher-bound block match satisfies that block commitment; it is not represented as a completed whole-file `x` comparison. A locally bound index without a matching event commitment authenticates continuity with previously retained bytes, not the publisher's intended file; it cannot satisfy a policy requiring publisher-authenticated content. No policy infers authenticity from decoding success. With insufficient commitments, the authenticated policy waits for a supported complete verification path or reports unavailable verification; it does not silently downgrade to improve latency.

# 3. Prior-work boundary

Dashlet supplies the useful idea of swipe-aware ordering under uncertain play-start times. TLadder supplies the architectural idea of optimizing the available choice set using playback feedback. Their published descriptions support those inspirations; neither establishes the correctness or performance of WARP's heterogeneous-origin controller. [S19, S20]

This specification does not inherit optimality guarantees from bitrate-ladder optimization, adaptive submodularity, or bandit theory. It also does not make the supplied manuscript's numerous secondary research references implementation dependencies. The core must work before those optional methods are evaluated.

# 4. Architecture and ownership

```text
Nostr events --> normalizer / evidence ledger --> presentation + retrieval DAG
                          ^                               ^
optional bound maps --> validator --> local index/compiler
                                                          |
player telemetry --> fast safety controller --> scheduler + calendar
                                                          |
                       atomic resource leases + single-flight broker
                                                          |
                          validated fetcher + applicable verification
                                                          |
                          sparse / complete encoded-byte store
                                                          |
                  native source / gateway / local virtual packaging
                                                          |
                            demux / decode / presentation backend

Measurements --> bounded watch/network/capability models --> scheduler
Snapshots    --> optional digital twin --> revalidated plan proposals
```

## 4.1 One external transfer owner

All controlled media requests—including sidecars, authentication blocks, manifests, initialization, audio, posters, virtual-source misses, retries, and prefetch—MUST pass through the request broker. A player library's loader either delegates to this broker or uses generation-scoped gateway URLs. Hidden independent downloaders defeat deduplication and budgets.

A native player may request a very large or open-ended range. That is a delivery request, not an immediate deadline for every byte. The broker splits upstream work into bounded milestones using current contiguous buffer and the adapter's need map, preserves downstream byte order, and limits speculative read-ahead. An active response does not receive unlimited emergency priority merely because it belongs to the active player.

There is also exactly one **representation-selection authority** per presentation. A mature HLS/DASH adapter may run its own ABR within WARP's byte/buffer/quality envelope; alternatively WARP may select representations and disable competing adaptation. The ownership is explicit. Adaptive libraries expose configuration for these choices; their defaults are not the WARP contract. [S18]

## 4.2 Fast path and planning path

The fast path handles player demands, invalidations, cancellations, urgent lease changes, and already-known cache hits without waiting for a model update, parser subprocess, database transaction, or digital-twin run. Background planning works from immutable snapshots.

All requests and callbacks carry action, generation, and playback-epoch IDs. A completion for a previously swiped-away item may update its own cache/model state, but MUST NOT mark the new active item ready.

The gateway requests a demand from the broker; it does not independently start unbudgeted origin fetches. A demand inherits the earliest deadline among its consumers. Priority inheritance continues through parser and disk work so an urgent media miss cannot remain queued behind background cache jobs.

# 5. Evidence, HTTP correctness, and progressive discovery

## 5.1 Nostr normalization

Support NIP-71 regular video kinds `21`/`22` and addressable kinds `34235`/`34236`. Preserve the `d` identity and exact event revision for addressable items. Parse each `imeta` entry at the first separator, preserve repeated fallback/image fields, and bound tag counts and lengths. NIP-71 also describes separate audio tracks; the normalizer must not interpret every `imeta` entry as a video resolution. [S1, S2]

Preserve unknown fields for forward compatibility within a size limit. Conflicting duplicates are evidence conflicts, not instructions to fetch every value. URLs, durations, sizes, dimensions, codec strings, bitrates, and hash syntax require bounded parsing. Do not fetch an original source merely because `ox` is present.

Blossom server-list discovery is an optional endpoint-discovery path using the author's `kind:10063` list. Bound fan-out, cache the list, and validate every generated URL. A hash-shaped pathname remains an identity claim until checked against downloaded bytes. Do not enumerate arbitrary servers or turn discovery into a crawl. [S4, S5]

## 5.2 Evidence and verification

Store field observations as:

```text
(value, source, event_revision, endpoint_context, generation,
 observed_at, model_epoch, uncertainty_or_conflict)
```

Observations of actual bytes outrank filename hints for structure. A complete digest comparison outranks a declared digest for byte verification. Do not average contradictory object sizes or codecs into a fictional value. Keep transient endpoint facts separate from immutable facts about retained local bytes.

Use discrete verification states:

```text
UnboundBytes
ExpectedDigestUnverified(expected_sha256)
CompleteLocal(computed_sha256, expected_match: absent | true)
VerifiedRanges(trusted_chunk_map_id, intervals)
Mismatch(expected_sha256, computed_sha256, provenance)
```

`CompleteLocal` with no expected hash permits local byte deduplication but does not authenticate a publisher's intended file. `VerifiedRanges` requires an established authentication binding, such as the optional `WRM/1` mechanism in Sections 5.8–5.9 or a local chunk map derived from a previously verified complete object. It is not obtained merely by hashing each received block locally. Keep the bound block-map identity distinct from any still-unverified complete-file digest claim.

Readiness and endpoint reliability may have calibrated probabilities. Cryptographic match status does not.

## 5.3 Representation generations and range assembly

A provisional generation key includes the exact resolved resource identity, request-selection context, authorization partition, content coding, and strong version evidence where available. Preserve signed query parameters and order; do not “normalize” different access URLs into one cache key. HTTP `Vary` also affects reuse. [S7]

**Default unauthenticated assembly policy:** merge network responses only for the same selected resource/context with compatible strong-validator evidence. A matching strong ETag is origin/resource-scoped, not a cross-origin identity credential. In the first implementation, weak ETags and dates are not resume credentials. Handle an omitted response validator after a valid conditional request according to its conditional semantics rather than inventing a new version. [S6, S7]

Without adequate version evidence, use one continuous response, an alternative endpoint, complete-object acquisition before reuse, or the independently verified authentication-unit path in Section 5.9. This conservative fallback may cost more bytes on weak servers; measure that cost. Do not conceal optimistic multi-request assembly behind the word “verified.”

Different endpoints claiming the same `x` are **candidate mirrors**. They MUST NOT contribute unauthenticated ranges to one trusted sparse object. Either verify independently complete objects, use authenticated chunks, or keep their provisional generations separate and restart the playback source at a valid boundary. HTTPS and past successful verification are not proofs of newly received partial bytes.

A mismatch quarantines the implicated generation and provenance chain. It does not invalidate a known-good complete local object or automatically ban every endpoint associated with the claimed digest.

## 5.4 HTTP execution contract

Request byte-addressed media with identity content coding and explicitly disable transparent decompression in the HTTP client. Content coding changes the byte address space; transport framing is not content coding. The native fetch client must expose the received status, relevant headers, and actual body progress. For example, reqwest provides explicit decompression, redirect, and retry configuration; pin and test the chosen version rather than relying on defaults. [S6, S16]

Use single-range upstream requests initially. The executor implements this state table:

| Observation | WARP transition |
|---|---|
| Coherent `206`, supported byte range, consistent version/context | Store exactly the received interval provisionally; compare it with the requested interval and framing. |
| `206` with an unknown total length | Use known offsets where valid, but do not infer EOF, a full-object size, or suffix-serving ability. |
| `200` after a range request | Treat body offset as zero. Re-evaluate a new full-response path; never write it at the requested nonzero offset. |
| `416` with a usable length indication | Reconcile size/generation evidence and replan; do not repeat the same unsatisfiable range indefinitely. |
| Truncation or interrupted body | Record only actually received bytes as incomplete. Never publish a complete interval or object prematurely. |
| Conflicting length/version or malformed range semantics | Invalidate the affected plan; isolate the generation and select a bounded fallback. |
| `401`/`403` | Authorization/policy outcome; do not synthesize credentials or hammer other authenticated endpoints. |
| `404`/`410` | Negative-cache this endpoint/resource within policy; allow bounded alternative discovery. |
| `429` or transient service failure | Respect applicable retry guidance, use jittered backoff, and suppress speculative retries. |

The detailed HTTP rules remain those of RFC 9110/9111; this table defines the controller's reactions, not a replacement HTTP parser. Use an established framing implementation and test the broker's range mapping independently. [S6, S7]

Reuse a bounded connection pool instead of creating a fresh HTTP client per action. Include connection setup and cancellation/restart costs in completion estimates. Optional connection warming is restricted to high-reach near-term endpoints and charged to request/privacy budgets; it is not a substitute for useful media preparation. Cancelling one transfer should preserve unrelated healthy transfers when the transport supports that isolation.

HEAD is optional, and only valuable when it changes the next decision enough to justify its latency. HEAD has no media body. `Accept-Ranges` is advisory; a request outcome is evidence about that request, not a permanent promise about an origin.

## 5.5 Correct in-flight continuation

The planner may raise the allowed read budget of an **already-open response that actually contains the desired future bytes**. It cannot extend an HTTP response beyond its response range.

For example:

```text
Request:  Range: bytes=0-65535
Response: 206; Content-Range: bytes 0-65535/358400
Result:   The response ends after 65,536 bytes.
          The remaining 292,864 bytes require a new request.
```

A `200` response starting at zero can be continued to EOF if admitted. A `206` can finish an object only when its actual interval plus compatible existing bytes covers the entire object. A bounded prefix that already covers a tiny complete object needs no follow-up. A HEAD-to-GET transition is a new request. Connection reuse is not response reuse. [S6]

For resumable data, issue the missing interval under suitable version conditions. When a conditional range returns a new full response, handle it as a new generation; never append it blindly.

## 5.6 Discovery policy

The default decision order is:

1. Reuse admissible local indexes/evidence/bytes and remove impossible backend choices.
2. Compare a cancellable streaming GET, a bounded prefix request, and any eligible map-assisted route to the same readiness target. Use a trusted-enough size estimate only to predict cost—not as an enforcement mechanism.
3. Add HEAD only where learned behavior or a specific decision gives it positive value.
4. Classify actual bytes and ask the container adapter for the next useful offsets/dependencies.
5. Continue toward startup or complete-object readiness only under renewed byte/time/storage leases.

A direct streaming GET does not need a known total length: it needs a bounded read lease and a stop policy. Conversely, a claim that the whole object will complete before a deadline does require credible remaining-size and completion estimates. This separates **permission to start** from **confidence of full completion**.

Unknown size, ignored ranges, slow body delivery, and oversized metadata all have independent byte, elapsed-time, idle-time, allocation, and request-count caps. Cancellation is asynchronous and can interrupt a slow body read; it MUST NOT wait for a 128–512 KiB progress block to fill.

## 5.7 ISO BMFF discovery and exact byte dependencies

Use a bounded top-level box iterator and established media parsing components. The iterator tracks absolute offsets, supports ordinary/extended sizes, checks overflow and file bounds, and makes forward progress. When an `mdat` box has a usable size, the next box offset can be calculated without downloading the payload. Prefer such structural jumps over repeated large tail windows. MP4Box.js is an example of a parser interface that explicitly returns the next useful input offset. [S11]

Suffix requests can discover a tail without a preceding HEAD or known object length. A bounded geometric tail search remains a fallback when structural navigation is unavailable. Fetch only missing portions of expanding windows where version evidence permits reuse. Finding the four bytes `moov` inside arbitrary media payload is not sufficient; validate the candidate's size, nesting, contents, and relation to the file layout. [S6, S11]

A front `moov` is not a readiness certificate. The adapter must determine track configuration, presentation timing, random-access dependencies, sample sizes, and sample/chunk offsets. Poor interleaving may put startup audio and video far apart. Average bitrate times seconds is a planning estimate, never the authoritative range map.

The W3C MSE ISO BMFF format defines a fragmented byte-stream contract; it is not a claim that arbitrary classic MP4 prefixes can be appended to `SourceBuffer`. Native MP4 delivery preserves original offsets. An MSE route uses a compliant fragmented representation or an explicitly admitted transmuxer. [S10]

## 5.8 Compiled readiness indexes and map trust

**Local compiled indexes are core functionality.** Retain a bounded result of successful structural analysis separately from bulky media ranges when retention policy allows it. The key includes the source binding, selected tracks, parser/index-format version, and relevant backend profile. Keep exact source time bases, sample offsets/sizes, decode and presentation timing, random-access dependencies, and initialization locations. Evicting media does not automatically invalidate structural facts about an immutable source; changing the source binding or parser semantics does.

An index compiled from only origin-versioned provisional bytes remains origin/generation-scoped. Knowing a claimed `x` is not enough to reuse that index against an arbitrary new response. An index derived from a complete locally hashed file can carry a locally generated block map. Newly fetched blocks can then be checked against that map without downloading the complete file again, provided access and retention policy still allow the use.

**A published readiness map is a shortcut, not a decoder or a source of authority by itself.** `WRM/1` describes bounded resources, presentation/track choices, initialization spans, and playable-interval dependency closures. Optional block hashes support independent verification. The wire format and exact commitment tag are in Appendix D. No executable recipes, codec plugins, arbitrary headers, credentials, or parser commands are accepted from the map.

Map status has two independent dimensions:

| Dimension | States and meaning |
|---|---|
| Authenticity/binding | `Hint`, `PublisherBound(event_revision, map_digest, primary_digest)`, or `LocallyBound(source_digest, local_index_record)` |
| Structural validation | `Unchecked`, `SchemaChecked`, or `AdapterValidated(parser_version, profile, validated_scope)` |

There is no remote `trusted: true` field. A third-party signature establishes that third party's statement, not the publisher's authorization. `SchemaChecked` rejects malformed descriptions but does not establish that the described samples actually exist or decode. Even a publisher-bound map can be structurally wrong. The adapter validates the initialization and relevant media facts before issuing a readiness certificate. It may fetch the map's proposed dependencies in parallel and thereby avoid serial discovery; it does not skip parser bounds or decoder checks.

An untrusted hint may propose offsets on an already eligible source within the discovery budget. It MUST NOT establish object identity, open unrestricted new origins, authorize cross-endpoint assembly, replace validated structure, or assert readiness. An invalid map is negative-cached by its own digest/context; the media's ordinary retrieval route remains available.

## 5.9 Independent verification and cross-endpoint progress

`WRM/1` deliberately uses a bounded flat SHA-256 block list rather than a new Merkle-proof protocol. For short finite files this provides a simple interoperable first implementation. Authenticated block trees are an established alternative, as illustrated by BitTorrent v2, but WARP does not adopt that transport or claim wire compatibility. A paged/tree map requires a new version and explicit proof verification; it is not an implicit fallback. [S24]

For a bound map and resource of length `L`, block size `K`, and index `j`, verify exactly the bytes in `[jK, min((j+1)K,L))` against hash entry `j`. The final block is not padded. All block bytes must be available before verification succeeds. Fetching a 5 KiB sample from a 64 KiB authenticated block may require the entire block; that amplification belongs in the request, data, memory, and readiness estimates.

The fetcher stages bytes by endpoint attempt. The verifier emits a verified-block record containing `(trust_anchor, map_digest, resource_id, block_index, exact_length, digest, provenance)`. Only then may the block enter a shared authenticated resource view. A missing block can be obtained from another eligible endpoint without discarding verified siblings. Deduplication uses the authenticated resource/map binding and block index, not URL or hash-shaped filename alone. Ordinary sparse generations and authenticated resource views remain distinct namespaces.

A block match means **matches this bound block commitment**. It does not mean the full resource has already matched the map's whole-file SHA-256 or the NIP-94 `x`. Once every byte is retained, hash the complete resource in order when committing it as complete. A publisher can commit inconsistent full-file and block hashes; retain that distinction instead of trusting one comparison to stand in for the other.

Failure handling is provenance-specific:

- A bad block from one endpoint quarantines that attempt/block provenance; retain independently verified siblings and healthy alternatives.
- A complete resource that satisfies every bound block but disagrees with the committed full-file digest is a **map consistency failure**, not proof that every endpoint is bad. Disable that map/resource binding for new use, invalidate affected readiness claims, and select the ordinary path or report a typed failure. Already presented frames cannot be retracted.
- A same-origin strong-validator violation still invalidates that origin generation. It cannot overwrite a previously verified block.

Without block hashes, a bound per-resource digest can authenticate that complete resource after it is downloaded. This is often sufficient for small initialization files and separately hosted segments. A manifest digest still does not authenticate its descendants unless their individual commitments are included and checked. Map-authenticated media can be used without strong HTTP validators **only after the required authentication units verify**; unauthenticated partial bytes never inherit that exception.

## 5.10 Map acquisition and invalidation

A map is another budgeted candidate action. Compare `map retrieval + verification + useful media` against ordinary discovery to the same readiness target. Prefer a cached/local index; do not impose a mandatory sidecar request on every video. A cold map request may run alongside a bounded ordinary path only when duplicate discovery cost is admitted. The first validated useful path wins; shared demands survive correctly.

Read map bytes with identity content coding under a strict map cap, verify the raw-body digest against an already authorized commitment, then parse bounded UTF-8 JSON and validate the schema/semantics. Digest mismatch, unsupported version, conflicting commitments, timeout, or excessive structure disables this shortcut, not the post. No map follows another map; `WRM/1` has no recursive includes. All resource URLs pass the normal endpoint gate.

The signed event commits to the map digest and primary resource digest. The map deliberately contains neither its own digest nor the final event ID; this avoids a circular hash dependency. Publish the media, then the map, then the signed event. Multiple URLs for identical committed map bytes are replicas. An addressable-event update changes the authorized mapping for new selections; an active playback epoch may finish its already pinned authorized generation unless revocation/logout policy requires immediate stop.

Store authority outside the map. Removing a user authorization or expiring a private access context invalidates access even when the content hash remains known. Apply privacy and `no-store` retention conservatively to derived indexes as well as payload. A cached index is not permission to reacquire private media after logout.

# 6. Retrieval ladders and compatible playback plans

## 6.1 Plans by media class

| Media condition | Useful target | Required path |
|---|---|---|
| Front-metadata MP4 | Selected-track startup samples and continuation interval | Native progressive streaming or valid sparse ranges; measured decoder preparation |
| Tail-metadata MP4, version-coherent ranges or authenticated blocks | Initialization plus necessary media intervals | Cached index/map candidates, structural jumps/tail discovery, safe sparse delivery |
| Fragmented MP4 | Initialization plus a valid random-access fragment and selected audio | A backend-compatible native or MSE adapter |
| Supported WebM | Initialization/track metadata plus suitable Cluster/random-access data | Tested native or WebM MSE adapter; no MP4-specific assumptions [S12] |
| HLS VOD | Selected playlist/dependency closure plus startup audio/video | Bounded manifest graph and tested HLS adapter |
| DASH VOD | Selected MPD/dependency closure plus startup audio/video | Optional tested DASH adapter; report exact supported profile |
| Nonseekable stream with early usable initialization | Sequential useful playback | Keep one streaming response and manage bounded read-ahead |
| Initialization usable only at EOF, no safe sparse path | Complete object, then decode | Bounded whole-object acquisition or defer |
| Compatible codec, incompatible packaging | Backend-compatible virtual fragments where supported | Locally compiled `VPK/1`; materialized remux only if the complete route is worthwhile |
| Unsupported codec | Alternate compatible variant, or optional transcode | No claim that remuxing changes codec support |
| Unresolved structure | Further bounded discovery or defer | Do not mislabel as malformed or definitively full-only |

## 6.2 Manifest resources form a graph

Treat a manifest as a dependency graph, not a small self-contained video. A playable path may require a master and media playlist, initialization, selected audio, media segments, and a decryption resource in profiles that support encryption. Resolve relative references against the containing playlist's effective URL; process every descendant URL through the same security and budget gate. HLS defines relative references, initialization mappings, byte-range segments, and multiple renditions. [S9]

Bound total manifest bytes, recursion, descendant count, URI length, variant count, and refresh requests. Reject cycles and unsupported features with a typed reason. Use parser-specific safe XML settings for DASH, including no external entity/network resolution.

A hash of an HLS/DASH manifest verifies only that file's bytes. It does not authenticate referenced segments or keys. Identical manifest bytes on two origins can resolve relative URLs to different resources. Include manifest generation and effective base URL in the presentation dependency context.

Rewrite supported references to controlled resource IDs or use a custom loader. Rewritten manifest bytes are a derived object with a new digest; the original manifest's `x` must not be advertised as their digest. Reject unsupported externally fetching constructs instead of forwarding an ungoverned URI to the player.

## 6.3 Quality and switching

Choose the lowest-cost variant that meets a configured perceptual floor for the viewport and device, then upgrade only when continuation safety and budgets allow it. Use codec support, dimensions, frame rate, decoded drops, and power/smoothness hints. A no-reference quality model is optional; do not substitute bitrate or resolution for proven perceptual quality. Media Capabilities can provide useful device hints, but actual playback still supplies the evidence. [S13]

During an active presentation, switch only where timeline, random-access, initialization, and audio continuity are compatible. NIP-71 alternatives or common `ox` alone do not establish these properties. Otherwise perform a controlled source restart/seek and record the discontinuity; do not promise seamless cross-file ABR.

Downloading more of an unchanged encoding improves coverage or cache availability, not intrinsic visual quality.

## 6.4 Pruning without false guarantees

Use **structural pruning** confidently: an action that fetches an already available interval, or an equivalent dependency plan with strictly greater deterministic cost and no extra information/reuse, can be removed.

Other pruning is heuristic unless the plans have equivalent state transitions, dependency effects, admissibility, information, resource interactions, and uncertainty ordering. Better means, p95/p99 values, or on-time probabilities at a few deadlines do not prove dominance for every objective or future state. Keep plans with materially different information outcomes, provenance, congestion exposure, or resume behavior.

Limit candidate count using explicit buckets and report how much approximation is introduced. Do not claim global Pareto-optimality or inherited TLadder guarantees.

## 6.5 Virtual packaging versus materialized transforms

Treat these as three different operations:

| Operation | Payload processing | Admission |
|---|---|---|
| Direct passthrough | Original representation, unchanged | Preferred when the tested backend already plays it efficiently |
| Virtual packaging (`VPK/1`) | Generate headers and expose references to original encoded samples; no re-encoding and no required whole-output file | A first-class bounded retrieval rung when the supported path is cheaper |
| Materialized remux/transcode | Write another packaged representation, or decode/re-encode media | Background or explicit-intent work with full scratch/output/CPU reservations |

Virtual-file systems can represent a remuxed file using generated metadata and source-sample references, producing payload only when read. Meta's published description supplies precedent for that architecture, not performance evidence for WARP. GPAC's progressive parsing/segmentation interface supplies an implementation example for incremental MP4 processing. WARP's narrower supported subset and contracts below are design choices. [S23, S11]

### 6.5.1 `VPK/1` scope

The first virtual-packaging profile converts a supported finite classic MP4 into **locally generated initialization and fragmented-MP4 segment endpoints** for a tested MSE/native-fragment backend. It does not promise arbitrary virtual monolithic MP4, arbitrary codecs, or generic composition of unrelated files. Direct native original-offset playback remains a competing route.

The minimum enabled codec subset is length-prefixed AVC with stable decoder configuration, plus optional AAC-LC audio, only on backends that actually support those configurations. Preserve decode timestamps, composition offsets, sample durations, and exact sample payloads. Accept B-frame composition offsets only in a tested implementation that writes the corresponding signed/unsigned fragment fields correctly. Initial implementations reject encryption, external data references, changing sample descriptions, nontrivial edit lists, unsupported priming/roll-recovery behavior, and unproven open-GOP starts. A sync-sample flag alone is not proof of an independently decodable boundary. Unsupported files fall back to native playback or another variant; they are not labeled corrupt.

The compiler requires enough validated initialization/sample metadata to determine the requested fragments, codec configuration, dependency closure, and output lengths. A `WRM/1` interval hint is not a replacement sample table and cannot directly generate fragment headers. When source metadata is unavailable until EOF and no safe range path exists, virtual packaging does not remove that acquisition dependency.

### 6.5.2 Locally compiled output plan

For each initialization object or media fragment, construct an immutable concatenation plan:

```text
VirtualObject {
    id: local_generation_id,
    compiler_profile_and_build,
    source_bindings[],
    parts: [Generated(bytes) | SourceSlice(binding, [start,end)), ...],
    exact_output_length,
    plan_digest,
    optional_computed_output_digest
}
```

Output offsets are prefix sums of part lengths. Validate additions for overflow, source bounds, part-count limits, and maximum output size. `Generated` bytes come only from the local validated compiler; remote maps cannot inject them. Source slices refer to fixed generation or authenticated-resource bindings, never mutable bare URLs. A slice may be served through bounded scatter/gather buffers; “virtual” is not a claim of zero copies in the OS, language runtime, or media backend.

The output identity is a local immutable representation identity. A hash of the plan is **not** the SHA-256 of the bytes it produces. Only an actual hash of all emitted bytes can be advertised as an output content digest. A local strong validator may identify a frozen plan and fixed source bindings, but on binding failure the response fails—it must not substitute bytes from a new source generation.

### 6.5.3 Fragment construction and serving

The compiler emits initialization and media segments conforming to the selected byte-stream format. In particular the ISO BMFF MSE contract includes fragment-relative addressing, track decode times, and media data covering referenced samples. Implement this with a tested muxing component, not ad hoc concatenation of arbitrary MP4 ranges. [S10]

The local compiler MUST:

1. Map selected source samples to exact decode/presentation timelines, including required audio and preroll within the supported profile.
2. Choose a real random-access boundary for startup/seek and identify every referenced sample and configuration dependency.
3. Freeze generated boxes, fragment numbering, field widths, offsets, and source spans before exposing a fragment URL or length. Split an oversized fragment or reject the profile rather than overflowing an offset field.
4. Obtain leases for generated metadata, staging buffers, source authentication units, and backend append/activation work. Fetch only the admitted missing source union through the broker.
5. Emit bytes in output order under backpressure. Keep sources pinned for active readers. The existence of a descriptor does not mean its payload is locally ready.

`VPK/1` defaults to one track per output fragment endpoint with synchronized audio/video append control; a muxed variant is a separately tested backend option. Generating a full classic-MP4 virtual view is deferred because it needs a stable whole-file layout and can induce scattered reads across much more source data.

A request intersecting several virtual parts is split into corresponding generated-byte slices and source demands. Overlapping source demands deduplicate with native/parser/prefetch consumers. If a source fails after headers are committed, terminate the response as incomplete and report the typed failure. Never zero-fill holes, silently modify the plan, or turn it into a different video.

### 6.5.4 When virtual packaging is worthwhile

Compare complete routes to the same playback target: direct native playback, virtual fragments, another compatible rendition, or complete acquisition. Include metadata discovery, generated-header work, source read scattering, authentication overfetch, output buffering, decoder cost, and extra HTTP requests. Virtual output order does not rearrange the physical origin file; poor interleaving may still be expensive.

Preserve encoded payload once where practical. Cache small generated headers/descriptors separately and materialize output fragments only when measured reuse justifies duplicate storage. Do not pin an entire large source merely because a virtual descriptor references it; acquire bounded read leases on demanded intervals. A reusable descriptor can survive payload eviction but has to reacquire valid source bytes before use.

### 6.5.5 Materialized transforms

Prefer a compatible original, virtual path, or alternate variant before local transcoding. Remuxing changes packaging without inherently improving visual fidelity; transcoding changes the encoding and can reduce fidelity. FFmpeg's `faststart` is a second-pass metadata relocation, not a free startup primitive. [S14]

Materialized work requires bounded input availability, output/scratch reservations, CPU/energy limits, validated output, and positive expected reuse or explicit user intent. Store `(input_binding, exact_transform_profile, tool_build, computed_output_digest)`. A recipe does not prove identical output across builds or hardware. Media workers receive governed byte sources, never arbitrary network-capable URLs. No transform may consume resources reserved for a feasible urgent playback path.

## 6.6 Cooperative startup-oriented publishing (`PUB/1`)

`PUB/1` is an optional uploader profile. Ordinary Nostr content remains on the heterogeneous path. NIP-71 can describe variants and separate audio, but common event membership or lineage does not establish aligned switching; the publisher and client must validate that property separately. [S1]

### 6.6.1 A real baseline, not a disposable preview

Publish at least one compatible **baseline rendition covering the full clip** at an acceptable quality floor. Its first interval is the beginning of that rendition, not an unrelated preview, a speed-modified rendition, or a dead-end teaser. When a higher-quality upgrade is late, the baseline remains a valid continuation choice. This is a property of packaging and availability policy—not a guarantee that its origin cannot fail.

The minimum interoperable publishing target is ordinary clear HLS VOD with a finite baseline playlist, initialization where required, and complete media segments. A progressive-MP4 alternative is useful for clients without that HLS profile. Both remain standard playable resources without WARP sidecars. Include accurate MIME, dimensions, duration, file digests and fallback locations where available; compute hashes after final packaging.

The publisher measures the **startup dependency footprint**: manifest chain, initialization, actual first random-access interval, required audio, and authentication overhead. Minimize that footprint and unnecessary serial fetches at the chosen quality floor. Nominal average bitrate, front `moov`, and short nominal segment duration are insufficient certificates.

Use early initialization, an inexpensive supported opening random-access point, aligned audio/video start, and bounded startup segments. Shorter GOPs/segments and lower bitrate can cost coding efficiency, quality, or request overhead; the publishing profile is a trade-off, not “shortest segments everywhere.” One to two seconds is an initial experiment range, not a format requirement.

### 6.6.2 Optional higher-quality renditions

A switching-enabled publication MUST preserve presentation identity, duration/timeline meaning, compatible random-access boundaries, selected audio continuity, and supported codec reconfiguration rules. Validate switches in both directions on the target backends. HLS has rendition/timestamp requirements, but a declaration is not a tested decoder transition. [S9]

The player starts the chosen baseline only when its startup closure and continuation plan meet policy. It upgrades at an admitted safe boundary after the upgraded dependency closure is ready and retaining the baseline path is still budget-feasible. If the upgrade misses, keep the baseline; do not suspend playback while waiting for better quality. The initial default is **no mid-clip switching** unless that backend/publication combination is certified. Selecting a suitable rendition once per short clip is a valid `PUB/1` client behavior.

Do not keep downloading all of both renditions “for safety.” Retain only the bounded fallback window justified by the switching plan, and cancel obsolete consumers at the committed boundary.

### 6.6.3 Asymmetric replication

Prefer replicating ordinary small objects—baseline playlists, initialization, and the first complete segment(s)—rather than inventing a mandatory archive format. These startup resources remain usable by existing players. A cooperating client may additionally mirror authenticated blocks from a monolithic representation, but such a block is not published under the whole-file hash as though it were the entire object.

Replicate on distinct permitted hosts only under the uploader's explicit publication, data, storage, and privacy policy. Verify replicas before publishing availability claims; publish stable complete-resource identities. A map may list replica locations, but each endpoint still passes the client's security and authorization gate. Prefix replication does not imply that full continuation exists on every prefix host; list actual full-resource/continuation locations and resolve manifests correctly.

For illustration, 512 KiB is 2% of 25 MiB, so another 512 KiB startup copy uses 2% of another full 25 MiB copy's payload storage. This arithmetic excludes metadata and extra rendition storage and is not a measured saving. Prefix-oriented caching has prior literature; the WARP adaptation is authenticated, dependency-complete startup resource replication rather than arbitrary byte prefixes. [S26]

The retained startup interval must bridge to a plausible continuation acquisition path under Section 7.5. A fast first frame followed by a predictable stall is not success. Each upload publishes the ordinary media first, then optional `WRM/1` sidecars, then the signed event commitment; failure to produce a sidecar must not block ordinary playback.

# 7. Watch, network, and capability models

## 7.1 Watch time and deadline distributions

Start with a small empirical survival model using duration buckets, session swipe pace, and a global prior. Add creator/video-specific components only with enough uncensored evidence. The model needs a distribution, not a single predicted watch duration.

Let `S(t) = P(W > t)`. For an uninterrupted normal-speed presentation already watched for `e` media seconds, with `S(e) > 0`, the remaining-watch survival distribution is:

$$
P(W-e>s\mid W>e)=\frac{S(e+s)}{S(e)}.
$$

Do not repeatedly sample a fresh complete watch time for the current video. Do not assume independent future swipes when a shared session state explains burst-swiping. Include session exit as an explicit transition so distant items do not all have eventual reach probability one.

Separate **media time consumed**, **foreground wall-clock dwell**, and **time waiting for playback**. Pauses, stalls, loops, seeks, backward swipes, and backgrounding are explicit events. A simple forward-only baseline may freeze/reset predictions on unsupported transitions, but must not treat a stall as useful content watching or as free extra time that improves its own objective.

For future item `i` and interval `u`, maintain a joint reach event `reach(u)` and need time `D_u`. A cheap scheduling heuristic is:

$$
R_u(c)=E\left[\mathbf{1}_{reach(u)}\min\{(c-D_u)^+,h_u\}\right],
$$

where `h_u` caps the relevant waiting interval within the planning horizon. This includes reach probability; conditioning on reach and then forgetting its probability overvalues unlikely future units.

Use a small set of joint user/network scenarios or coarse distributions. An ordinary convolution is valid only under the assumed independence and timeline model. Production deadlines use actual current playhead, selected playback rate, buffered coverage, and elapsed session evidence.

## 7.2 Origin and shared-path models

Maintain separate estimates for setup/TTFB, useful payload rate, failure type, and range/version behavior. Split by method and broad size/context classes, then shrink toward coarser priors. Avoid an enormous issuer × URL × ASN × method × hour × device table before data supports it.

Model the **shared device bottleneck** in addition to endpoint performance. If several requests each achieved a rate in isolation, those rates cannot all be assumed available simultaneously. Candidate completion estimates include queueing, current concurrency, bandwidth allocation, connection state, and already-active transfers.

Correlated failures matter. Two hostnames may share a provider or path; two different providers still share the device connection. Scenario sampling should preserve observed common-path shocks. A different hostname is not evidence of independent hedge success.

Reset or down-weight stale estimates after a network/interface change. Use bounded recent summaries and a simple degradation detector before introducing complex change-point algorithms. A request cancelled for user navigation is censored, not an origin failure; its unobserved full completion time is not a successful latency sample. Throughput from a tiny TTFB-dominated range must not stand in for sustained transfer rate.

## 7.3 Metadata reliability and exploration

Field reliability may help choose between discovery plans. It MUST NOT bypass URL safety, read leases, parser limits, or verification. Start with field/source-class counters; issuer-specific estimates are optional and require enough evidence. Metadata can be self-reported or copied, so source agreement is not automatically independent corroboration.

Exploration is bounded to eligible feed candidates and disabled during urgent playback, Data Saver restrictions, backgrounding, or resource pressure. Circuit breakers distinguish authorization, not-found, rate-limit, malformed-media, and transport outcomes. Recovery probes are jittered and rate-limited.

## 7.4 Capability and decoder feedback

Key capability evidence by backend and relevant software/device epoch. Use feature detection and bounded playback tests. Treat demux/decoder failures separately from network failures. Capture dropped frames, decoder initialization time, audio/video gaps, RAM pressure, and power constraints.

An endpoint can be fast while the device cannot decode its media smoothly. A successful server-side decoder test does not prove the actual user's browser/device can play the same stream.

## 7.5 Finite-horizon service-deficit buffering

A startup cushion must protect against the continuation trajectory, not just one fast next request. Use the following buffer-conservation test for a **fixed** presentation, initial playhead, dependency graph, and proposed continuation schedule.

Let `b` be initial contiguous usable media seconds. Let `C(t)` be the media seconds that uninterrupted intended playback would consume by future wall time `t`; at constant playback rate `r`, `C(t)=r*t`. A commanded pause contributes zero consumption. A network-induced stall MUST NOT be treated as an intentional pause that makes the plan look feasible. Let `A_pi(t)` be new contiguous usable media seconds made available beyond the initial buffered frontier by schedule `pi`, including completion of required tracks and backend processing. Previously downloaded but disconnected intervals count only when they become contiguous and usable. Count each newly unlocked interval once.

For a fixed no-seek/no-swipe trajectory over horizon `H`, uninterrupted playback is feasible exactly when:

$$
b + A_\pi(t) \ge C(t),\qquad 0\le t\le H.
$$

Consequently, for that fixed arrival trajectory:

$$
b_{\min}(\pi,H)=\sup_{0\le t\le H}\big[C(t)-A_\pi(t)\big]^+.
$$

This is a buffer-conservation identity, not a new probabilistic guarantee. Its assumptions matter. `A_pi` must count actual dependency-complete playable extension, not received bytes, the largest downloaded timestamp, or the sum of independent audio and video durations. It includes authentication, packaging, disk and decoder/append delays when those gate usability.

### 7.5.1 Event-driven calculation

For stepwise playable arrivals and constant rate, evaluate the deficit immediately **before** each arrival and at the horizon. Just before an arrival, consumption may be maximal while that arrival is not yet usable. At a tied timestamp, evaluate the left limit once, then apply all completions at that timestamp. Playback ending exactly when the next interval becomes usable is not a positive-duration stall. Use integer/rational time internally and explicit tie rules.

```text
available = 0                       # newly usable media duration
required = 0
for completion_time, added_coverage in time_ordered_arrivals:
    if completion_time > horizon: break
    required = max(required, consumption(completion_time) - available)
    available += added_coverage
required = max(required, consumption(horizon) - available)
return max(0, required)
```

An arrival may unlock a previously cached downstream interval, but only the adapter's contiguous dependency closure can determine `added_coverage`. For piecewise playback rates, add rate-change timestamps; for uncertain swipes, simulate the relevant intended sequences jointly instead of adding buffers from different videos.

### 7.5.2 Example and planning use

If new usable media arrives continuously at `0.8` media seconds per wall second while normal-speed playback consumes `1.0`, a 20-second horizon has a four-second maximum deficit. A two-second buffer lasts ten seconds under this simplified service model. An origin with an initial pause followed by faster service has a different deficit trajectory even if its mean throughput matches.

Changing the initial buffer, rendition, playback position, or acquisition schedule generally changes `A_pi`. Recompute the timeline for each candidate. Do not calculate one four-second requirement and blindly prepend four seconds while reusing a service curve for a different set of source samples. Limit `H` to meaningful remaining playback; do not require service after EOF.

The core controller uses conservative continuation scenarios, a minimum immediate cushion, and a bounded horizon. If the predicted deficit exceeds retained coverage, compare acquiring more useful initial coverage, a lower-cost compatible rendition, a better eligible endpoint, or explicit degraded playback. Do not fetch arbitrary bytes to satisfy a seconds-only target. Clip byte/RAM/data limits remain binding; do not silently clamp the buffer requirement and then call the clipped value safe.

### 7.5.3 Uncertainty and degradation

For uncertain joint service trajectories `omega`, calculate `M(omega)=max_t[C(t)-A_pi(t,omega)]^+` and evaluate the tail of **that maximum**, not independent percentiles for each request or each time. The empirical quantile of `M` is a model estimate. A statistical interval from simulated samples does not include unmodeled network failures; label it accordingly. Do not call a simulated quantile a lower-layer service guarantee.

Use horizon and reach weights from conditional watch evidence to avoid preparing the full remaining clip for every user. Keep immediate current-playback protection separate from speculative long-horizon utility. With insufficient service evidence, disable aggressive deferral, use conservative fixed minimum buffers and observed failures, and mark the risk estimate unknown. Fixed buffers still cannot guarantee arbitrary Internet service.

# 8. Objective and enforceable budgets

## 8.1 One objective, no duplicate penalties

The reference controller first enforces policy and feasibility. Within an admissible priority class, compare a single loss over horizon `H`:

$$
\begin{aligned}
L_H(\pi)=E[&aD_{start}+gD_{stall}+fN_{failed\ starts}+dD_{switch}\\
&+c_B B+c_S A+c_C C+c_R N_{requests}+c_E E_{energy}\\
&+c_{rank}D_{rank}-w_Q U_{quality}]
+\rho\,\mathrm{CVaR}_q(Z)-V_{terminal}.
\end{aligned}
$$

`B` counts newly transferred bytes, including sidecars, authentication amplification, retries, probes, duplicates, audio, and discarded data. `A` is encoded-cache byte-seconds; RAM and scratch disk are separately constrained. `Z` is total user-visible start/stall delay in a scenario. All coefficients convert their terms into the same loss units. Energy may initially be enforced as a mode constraint instead of estimated in the loss.

`V_terminal` estimates only benefit **after** the simulated horizon, with bounded credit. Reuse within the horizon is already reflected in fewer future requests/stalls. No separate unexplained cache bonus is added on top. Unused bytes are a required diagnostic, not a second charge for the same bytes unless the product intentionally defines an extra waste preference.

Use the conventional definition:

$$
\mathrm{CVaR}_q(Z)=\min_z\left[z+\frac{E[(Z-z)^+]}{1-q}\right],\qquad 0<q<1.
$$

For example, `q=0.95` addresses the worst 5% tail under the model. Quantiles, coefficients, and quality floors are policy settings, not universal constants.

The selected action's value is the counterfactual difference `L_H(no action / alternative) - L_H(action)`. When this difference includes resource, rank, risk, and reuse effects, do not subtract/add those effects again in a secondary score. Reserve survival is a constraint/priority, not an extra uncalibrated reward.

## 8.2 Count visible waiting once

Initialization and startup samples are dependencies of one startup, not two independent user-visible delays. Similarly, delays of later media units cannot simply be summed against their original unstalled deadlines.

For serial units with available times `C_j` and playable durations `d_j`, a simplified no-swipe simulation uses:

```text
P_0 = max(play_request_time, C_0)
startup_delay = P_0 - play_request_time
P_j = max(P_(j-1) + d_(j-1), C_j)
stall_j = P_j - (P_(j-1) + d_(j-1))
```

The full simulator also models swipes, playback rate, selected audio, and backend activation. This recurrence explains why the sum of `R_u` values is only an urgency approximation, not exact total rebuffering.

## 8.3 Resource leases

Every external request or transform requires an atomic lease for its next bounded milestone. A lease contains:

```text
owner / consumers, priority, generation, expiry,
body-byte allowance, request allowance,
RAM allowance, disk-growth allowance, optional CPU allowance
```

The broker accounts for already-used resources and all outstanding reservations before issuing another lease. An estimate such as “probably 350 KiB” does not authorize an unbounded body. Workers consume leases incrementally and request renewal before crossing the allowance. A `JIT/1` calendar reserves the bounded future protected demand's data/disk/request allowance at group level; per-read leases draw from that reservation rather than reserving it a second time. Cancelled consumers release only their unspent share. Include full authentication-unit staging, generated headers, virtual descriptors, and local output buffers where relevant.

Implement separately:

| Budget | Enforcement |
|---|---|
| Transfer rate/burst | Global weighted token service; per-origin concurrency/rate limits |
| Session/day/network-class data allowance | Persistent spent-and-reserved ledger; no refill just because time passed within the allowance period |
| Encoded disk storage | Actual allocated bytes plus reservations, metadata, journal, and temporary outputs |
| RAM/decoder memory | Bounded queues/buffers and backend-specific warm-player limits |
| Requests | Per item, endpoint, origin, session, and global fan-out/retry caps |
| CPU/energy | Bounded workers, cancellation, thermal/low-power restrictions |

A token bucket limits rate, not cumulative data. Cached local delivery must not be charged again as new Internet payload.

## 8.4 Honest accounting boundary

Application-level read admission can be strictly bounded. Exact carrier-billed traffic cannot be guaranteed from a userspace body counter: protocol overhead, retransmissions, kernel/library buffering, and bytes already in flight exist outside that boundary. Record body bytes and, where available, interface-level bytes separately.

Reserve headroom for measured cancellation overrun and transport overhead. Stop speculative work before the allowance is exhausted, stop renewal at the hard application boundary, and expose any measured overrun rather than hiding it. A product requiring a strict interface-level limit needs enforcement at that interface, not a claim about a Rust token bucket.

Retries, redirect hops, and library-internal replays must be disabled or made visible to this accounting layer. Unknown-length streaming receives successive bounded leases; it is not exempt.

## 8.5 Optional shadow prices

Adaptive prices are optional after the hard controls work. They tune long-run target utilization but do not enforce hard limits. Add a nonnegative dual price to an explicit base resource coefficient once; do not charge it in both the rollout objective and the action score.

Use dimensionally consistent, normalized feedback, price caps, anti-windup, and hysteresis. Reset/rebase after budget-mode changes. Compare against fixed prices at identical cumulative allowances before adding more adaptive controllers.

# 9. Online scheduling and reserve protection

## 9.1 Priority classes and modes

Maintain three operating modes:

- **Emergency:** active presentation at risk, an explicit seek waiting, or an imminent next presentation lacking viable readiness. Preempt expendable work and run the best feasible urgent recovery.
- **Safety:** current presentation is protected but the next intended presentation or reserve lacks startup/continuation coverage.
- **Normal:** the required playback envelope is healthy; consider economical continuation, modest lookahead, optional quality/reuse, or idling.

Add orthogonal policy flags: Data Saver, offline, background, budget exhausted, low power, and storage pressure. These flags can forbid speculation even in otherwise normal conditions. They do not change the meaning of a media failure.

## 9.2 Reserve is a joint trajectory property

A reserve consists of semantically eligible future presentations with valid dependency leases, expected activation delay, and useful coverage. A count of prepared posters or isolated first frames is not sufficient.

Define `F_Hr` as an event in a joint scenario: an intended/allowed presentation cannot start within the product's grace time, or visible playback underflows, before recovery horizon `H_r`. A calibrated advanced controller may require:

$$
UCB\big(P(F_{H_r}\mid state,policy)\big)\le\epsilon_r.
$$

This is a **model-based risk target**, not a physical guarantee. Statistical bounds over sampled rollouts do not cover model misspecification. Measure calibration by device/network/media strata and fall back to deterministic margins when evidence is weak.

A count quantile of future swipes is useful as a rough target only when the prepared items are reachable in permitted order and each supplies adequate coverage. Do not multiply marginal reach/readiness probabilities as though they were independent, or sum mutually exclusive future coverage and call it a survival guarantee.

The initial controller protects current contiguous buffer plus one intended next startup/cushion, then adds a small bounded reserve. It does not claim 99% reliability from sparse origin data.

## 9.3 Reserve usable service, not just a semaphore

Reserve the actual urgent dependency path: request admission, near-term tokens, disk growth, parser capacity if required, and decoder activation. Reduce background competition and use priority inheritance. Do not leave an empty “rescue slot” while avoidable urgent work is pending.

One reserved worker slot cannot guarantee bandwidth against an unresponsive server, shared connection congestion, or already in-flight traffic. Re-evaluate the completion estimate under the active allocation. On infeasibility, select the least-bad permitted recovery, preserve user intent, and report degradation. Never borrow from the user's forbidden data allowance to make a safety constraint appear satisfied.

## 9.4 Deterministic core policy

At each material event, and at the bounded periodic wake:

```text
apply event only to its matching action / source / map / player epoch
update leases, invalidations, consumer demands, and observed contiguous coverage

if offline, policy-limited, or allowance exhausted:
    stop forbidden work; serve retained permitted bytes; update honest UI state
    return

snapshot = active presentation + eligible future window + active resource use
look up valid local indexes; consider bound maps only as competing retrieval routes
compile next useful dependency closures, including authentication/packaging/activation
expand to missing source intervals/blocks; deduplicate shared jobs and consumers
classify urgency from actual input deadlines, next intent, and service deficit
remove unsafe, unsupported, duplicate, or unleaseable actions

protect feasible active/explicit-intent paths, then intended-next/reserve paths
calendar = build joint bounded reservations if JIT evidence/profile permits
for each available service opportunity:
    if a protected job is due or intentional deferral is not authorized:
        dispatch the most urgent feasible bounded path toward its useful milestone
    otherwise:
        fit economical optional work into genuinely spare time and resources, or idle
    recompute after each assignment to include contention and reservations
    acquire leases atomically, then dispatch

arm earliest latest-start / deadline / expiry / periodic timer
```

Use earliest downstream deadline and conservative slack among comparable urgent jobs. Deep current-video continuation is not all emergency work: only dependencies in its protected near-term envelope have that urgency. An imminent next start may precede later active-video bytes. Where several actions protect the same target, compare complete time-to-useful-state and incremental cost. For nonurgent work, a simple expected-loss reduction per service time is a labeled heuristic; all costs appear once.

Without a valid `JIT/1` calendar, the core does not intentionally delay needed near-term work based on a point throughput/watch estimate. It still avoids unneeded payload outside the protected envelope. Fill concurrent allocation iteratively with shared-path accounting; never assign several jobs the same unused bandwidth independently. Buffer-deficit evaluation and calendar invalidation are safety inputs, not optional bonuses in a score.

## 9.5 Adaptive sizing and direct fetch

Choose work units by media dependencies and cancellation latency, not a universal chunk duration. When exact byte ranges are known, fetch missing ranges around relevant random-access/track dependencies. Under authenticated-before-use policy, expand to complete authentication units before budgeting; virtual output demand is translated to its original source closure before this calculation. Coalesce nearby intervals only when saved request/setup cost exceeds overfetch cost and the larger request remains deadline/budget-feasible.

Compare direct acquisition and discovery using their **complete routes to the same useful readiness target**. A 20 MiB presentation with a cheap startup closure may beat a 1 MiB full-only object. A small object may favor one full GET over a prefix and follow-up. TTFB, uncertainty, active connection state, and likely abandonment all matter.

Fixed small-object and probe thresholds are bootstrap caps, not proofs of safety. Keep size belief separate from the runtime byte cap. Reuse already fetched compatible bytes; expanding a probe must not download the same prefix again accidentally.

## 9.6 Continue, stop, and resume

Already-transferred bytes are sunk; compare the remaining cost/benefit of continuing against the best alternative. Existing bytes matter through actual reusable coverage, reduced remaining work, and permissible future cache value.

A swipe removes that playback consumer. It does not cancel work still required by another consumer. The broker cancels only when no live demand or admitted cache-retention job needs the transfer. Hysteresis avoids repeated abort/restart near a threshold.

An open response may be read further only under Section 5.5. Pausing a read is not assumed to pause the origin at zero cost; bounded receive buffers, idle limits, and protocol behavior govern the decision. Nonresumable streams may favor a longer continuous read, but still cannot escape a hard lease.

## 9.7 Hedging and failover

Hedging is off by default. Enable only for an urgent small independently useful unit after evidence shows a tail benefit at an explicit duplicate-byte cap. Account for shared bottleneck and correlated-origin risk; stop the loser only after the winning data meets the required structural/verification policy, not on its first byte.

A same-hash claim does not authorize partial cross-origin assembly. Bound verified blocks may preserve progress across eligible endpoints under Section 5.9; race only missing authentication units and commit after verification. Whole-resource races without that mechanism keep provisional bodies separate. A representation/source change during playback goes through the adapter's supported random-access/timeline transition, not arbitrary append/splice.

For large failing transfers, a bounded missing-range retry on the same generation, a lower-cost compatible presentation, or a controlled source restart is usually a more suitable candidate than duplicating the whole download. Which wins is measured, not assumed.

## 9.8 Preserve semantic intent

The recommender owns candidate order and any semantic scores. Prefetch order may differ from display order; default display order does not change merely because another host is faster.

Optional rescue substitution requires explicit product policy: an admissible set, maximum displacement, grace interval, and starvation bound for deferred items. A score-difference threshold is used only when the recommender declares those scores comparable. Explicitly opened posts, searches, and backward navigation are not silently replaced.

Log a transport substitution separately from a user swipe/dislike. Never repeatedly hide an expensive item while pretending its low exposure reflects user preference. When no allowed recovery exists, show the intended item's poster and honest loading/error/retry state, not an unrelated video presented as that item.

## 9.9 Optional digital twin and lookahead

Only after the core is measured, add a bounded receding-horizon planner. It must model shared bandwidth, cancellations, decoder warm state, exact dependencies, and user transitions. It uses common random scenarios across candidate plans and applies the single loss in Section 8.

A probe changes later available choices. Evaluate a contingent policy that may choose different follow-ups after different observations; a fixed open-loop sequence is not a full value-of-information calculation. Either the rollout accounts for this decision value, or a separate greedy estimate does—not both.

Bound depth, width, scenarios, cache size, and wall-clock time. A result carries the snapshot epoch and is revalidated before execution. Missed computation deadlines use the deterministic core; playback cannot wait for the optimizer. The first deployable version does not need beam search, explicit CVaR estimation, or learned quality.

## 9.10 Staged commitment and latest-safe starts (`JIT/1`)

Separate **preparation** from **payload commitment**. High-reach next items may justify early event normalization, local-index lookup, bounded map acquisition, initialization, and one decoder warm-up. Larger continuation intervals remain deferrable until starting later would violate the protected playback schedule. Preparation itself consumes privacy, network, memory, and CPU budgets; it is not automatically free or always preferable to a direct GET.

For an isolated required job with need time `d` and a valid end-to-end service bound `l`, the latest start is `d-l`. This includes dispatch, queueing, setup, acquisition, verification, packaging, disk, and activation costs that lie on the critical path. It is **not** a license to assign every concurrent request its isolated rate.

### 9.10.1 Joint reservation calendar

The implementation MUST maintain a shared, expiring schedule for protected dependency jobs before intentionally deferring required payload. A calendar records:

```text
snapshot/network/model/player epochs; created_at; expires_at;
ordered deduplicated jobs with releases, durations, dependencies and deadlines;
request/origin service allocations; local-stage allocations;
byte/RAM/disk/request reservations; assumed service envelope;
latest permitted dispatch times; reason for each deferral
```

An economical first implementation uses a conservative **single virtual service lane** for all required work: order deduplicated jobs topologically, prefer earlier downstream deadlines among ready jobs, and include bounded setup, body, verification, and activation time in each job's cost. Serializing the whole path may reject feasible concurrent schedules; that is an acceptable conservative baseline, not proof of global infeasibility. Future multi-lane schedules must enforce shared network capacity and local resource constraints explicitly.

For a fixed topological job order, nonnegative releases and deadlines on one common monotonic time base, and positive service durations valid over the calendar horizon, compute a right-justified schedule:

```text
next_start = infinity
for job in reverse(fixed_order):
    finish = min(job.deadline, next_start)
    start = finish - job.service_duration
    if start < max(now, job.release):
        return INFEASIBLE_FOR_THIS_ORDER_AND_ENVELOPE
    reserve(job, start, finish)
    next_start = start
return reservations in forward order
```

All dependencies must be represented once and precede their consumers; propagated need times must include downstream processing. This recurrence preserves every job's deadline for the fixed serial model and avoids assigning two jobs the same service interval. It does not prove optimality for variable future capacity, different job orders, preemption, or parallel resources. A runtime service estimate is not a mathematical bound merely because the code names it `safe_duration`.

Already-active non-preemptible work occupies the calendar before new jobs. A pauseable body is only preemptible according to actual transport/cancellation behavior; allow for buffered/in-flight overrun and setup loss. Cancelled or shared jobs update resource ownership before replanning. Resource reservations for the bounded protected horizon prevent background work spending the allowance required by deferred jobs; leases for individual reads remain separately renewable.

### 9.10.2 Dispatch and invalidation

The fast controller dispatches the first due protected job, or performs optional work only when that work fits genuinely spare service/resources and cannot delay protected jobs beyond their starts. Network slack alone is not permission to consume protected disk or data reservations. It may intentionally idle while waiting for a useful job's latest start.

Arm a timer for the earliest of a job start, calendar expiry, a player deadline, lease expiry, or the maximum periodic control interval. Recompute on navigation, seeks, playback-rate changes, new buffer gaps, service degradation, network change, response/generation failure, metadata contradiction, decoder failure, cancellation, or budget pressure. An invalid calendar authorizes **no further intentional deferral**: fall back to immediate bounded protection, not to continuing an old sleep.

A current player's decoder/input deadline can be earlier than the visible buffer-underflow time. Honor the earlier deadline. Native-player demands whose true dependency horizon is unknown remain conservative; do not delay them using a guessed application buffer. Exact future swipe times are unknown: intended-next/reserve requirements use the configured joint scenario or conservative deadline envelope, not a point estimate of average watch time.

### 9.10.3 Conditional efficiency result

Assume the same required unit can finish in at most 300 ms at any admitted start, current buffer covers two seconds, and a 200 ms margin is reserved. Starting after 1.5 seconds still meets the two-second need time. If the user leaves at 0.7 seconds, deferral avoids a transfer an immediate policy may have completed. Under these fixed service and no-interference assumptions, waiting does not worsen the required playback deadline and can reduce abandonment waste.

The argument does not hold when early service is uniquely available, setup costs grow, a nonresumable response would be lost, future bandwidth is weaker, or the user navigates sooner than the protected envelope. Such states should favor early acquisition or disable `JIT/1`. The implementation MUST log prevented deferrals and missed latest starts, not report only successful byte savings.

# 10. Sparse cache, admission, and eviction

## 10.1 Store only committed intervals

The encoded-byte store maintains exact interval coverage, generation/provenance, verification state, HTTP reuse policy, and consumer leases. Disk holes are never served as zero-filled media. Incoming writes become readable only after their interval metadata and bytes are committed consistently; crash recovery must not advertise missing or partially written data.

Overlapping intervals are deduplicated. A changed version or inconsistent overlap invalidates the affected assembly rather than silently replacing a portion of a playing object. Bound interval-map cardinality and merge bookkeeping without filling real holes.

Count actual allocations, metadata, journals, scratch output, and outstanding writes against disk capacity. An untrusted multi-terabyte advertised length must not cause equivalent preallocation or unbounded interval structures. Pin exact needed intervals rather than entire large files whenever possible.

## 10.2 Single-flight demand and range coalescing

A broker table keyed by `(generation, resource/range)` tracks work already in flight. When player, parser, and prefetch demands overlap, attach consumers and fetch only the missing union. Every consumer has a deadline and cancellation reference. The highest urgency propagates to the shared request.

A coalesced request may include a bounded gap only under the overfetch policy. Cap waiters and pending ranges per object; otherwise a malformed player request or adversarial manifest can turn deduplication into unbounded memory use. Completion/error/cancellation must wake every affected waiter exactly once.

Do not charge speculative and demand consumers twice for the same external bytes. Record the original reason and later usefulness separately.

## 10.3 Admission and eviction policy

Start with a segmented recency policy: a small pinned playback/reserve region, probation for new data, and promotion on demonstrated reuse. Give initialization/startup **dependency bundles** a bounded preference. Do not admit a full object merely because its transfer finished.

A more advanced policy estimates marginal avoided cost over a **fixed future horizon**, divided by additional allocated bytes. Recompute value after other dependencies are evicted. An initialization range and the media it enables can have complementary value; independent per-range densities may destroy every playable bundle.

Do not divide by a residency duration predicted from the same eviction policy without a defined joint model. Do not count an expected future hit both in the current rollout and in terminal cache credit.

Under pressure, stop speculative completions/transforms first, then release unneeded distant ranges and lower-priority reserve. Active read leases protect bytes until readers release them. Pin budgets are finite: failed pin admission must cause replanning or degraded playback, not deadlock or disk overflow.

## 10.4 HTTP and privacy boundaries

Respect `no-store`, freshness/revalidation requirements, `Vary`, and authorization partitions. `no-store` media may pass through bounded transient playback buffers but is not admitted to reusable persistent storage. A hash in a public event does not override an endpoint's authorization or cache policy. Shared-gateway reuse requires a separate shared-cache policy. [S7]

A URL-only provisional generation is not a permanent content address. Complete local bytes can be keyed by their computed digest after verification/commit, while event and endpoint mappings retain their own freshness and authorization state. Deleting a mapping, logging out, or changing an event revision must release affected leases and access capabilities according to policy.

## 10.5 Knowledge, payload, and activation are separate caches

Maintain independently budgeted metadata/index storage, encoded payload, generated virtual headers/descriptors, and backend activation resources. Small validated indexes may survive payload eviction because they avoid future discovery. A descriptor reference is not an active pin on every source byte. Acquire read leases only for the dependency intervals currently needed; a bounded startup bundle can remain pinned without retaining the whole source.

Persist map authority and access scope outside untrusted map bytes. A schema-valid map is not automatically promoted to the trusted-index cache. Key interpreted state by parser/profile/build versions, and invalidate affected interpretations after upgrades without discarding still-valid authenticated raw bytes. Generated output payload is duplicate storage unless the implementation can actually share its backing bytes; account for real allocations, not a conceptual “zero-copy” design.

Give retained knowledge credit only for realistic future avoided acquisition/processing under current access policy. Do not keep every encountered sidecar indefinitely, convert map cardinality into an unbounded metadata cache, or treat a discarded video's decoder as cheap persistent state.

# 11. Rust implementation and frontend contract

## 11.1 Module boundaries

| Module | Responsibility |
|---|---|
| `nostr_ingest` / `evidence` | Bounded normalization, event revisions, declarations, conflicts, verification states |
| `media_adapter` | Backend capability, exact dependency maps, manifest/container parsing, readiness certificates |
| `index_store` / `readiness_map` | Local compiled-index reuse, optional `WRM/1` parsing, external authority binding, invalidation |
| `block_verifier` | Authentication-unit staging, full-resource/block checks, provenance-specific commit |
| `virtual_packager` | Optional local validated `VPK/1` compilation and bounded output mapping |
| `reservation_calendar` | Joint latest-start/resource schedule, service assumptions, epoch/expiry invalidation |
| `request_broker` | Demand union, single-flight, priorities, resource leases, cancellation references |
| `fetcher` | Validated DNS/redirects, HTTP semantics, exact progress, explicit retry policy |
| `cache` | Generation-aware intervals, atomic commit, leases, quota, admission, eviction |
| `scheduler` | Fast safety policy, deterministic milestone selection, semantic constraints |
| `models` | Small watch/network/capability estimates with bounded state and calibration |
| `gateway` / `native_source` | Controlled byte/segment delivery; no independent unbudgeted downloader |
| `player_bridge` | Playback epochs, actual contiguous buffer, activation/presentation/error telemetry |
| `repair_worker` | Optional isolated materialized packaging/codec conversions with explicit reservations |
| `publisher` | Optional `PUB/1` uploader packaging, replica verification, map generation and event commitment |
| `telemetry` / `replay` | Decision/outcome records, reproducible tests, optional planner snapshots |

One scheduler actor owns planning state. Bounded channels carry coalesced progress, urgent invalidations, and outcomes; critical cancellation and budget signals cannot be silently dropped behind telemetry. Network I/O, hashing, parsing, disk access, and simulation do not execute synchronously in the scheduler's hot path.

Use monotonic time for deadlines and durations; wall time for persisted observation age with clock-change handling. Resource leases use explicit ownership/release, including subprocess failure and task cancellation. Session restarts recover disk/accounting state before admitting speculative work.

## 11.2 Illustrative state contracts

These are interface requirements, not a complete Rust API:

```rust
// Internal intervals are half-open; HTTP byte endpoints are inclusive.
struct ByteSpan { start: u64, end_exclusive: u64 }

struct Demand {
    consumer_id: u64,
    playback_epoch: u64,
    generation_id: u64,
    span: ByteSpan,
    deadline: std::time::Instant,
    priority: Priority,
}

enum Priority { Active, ExplicitSeek, NextStart, Reserve, Background }

enum Continuation {
    ReadExistingBody { action_id: u64, extra_bytes: u64 },
    FetchMissingRange { generation_id: u64, span: ByteSpan },
    Stop { action_id: u64 },
}

enum FailureClass {
    Policy, Authorization, NotFound, RateLimited, Network,
    RangeSemantics, GenerationChanged, MalformedMedia,
    UnsupportedProfile, Decoder, Budget, Storage,
}
```

Constructors validate `end_exclusive > start` and checked conversions. An action record additionally identifies the actual HTTP response interval, its remaining readable bytes, and current lease; `ReadExistingBody` is invalid when those bytes are outside that response.

## 11.3 Gateway response behavior

An application asset ID may remain stable for UI purposes, but each byte-serving URL is **generation-scoped**. It must not silently switch to a different encoding, rewritten layout, or object length during playback. A new source requires a new URL/presentation epoch and a controlled player transition.

For a satisfiable supported single range, return the actual interval with coherent length metadata. A complete object GET returns the complete representation; a progressive GET streams the representation from offset zero rather than returning a short prefix as a successfully completed full object. A HEAD response has no body and must not invent an unknown length.

Support open-ended and suffix ranges when length is known. A known out-of-bounds request is different from an unavailable upstream interval: the former may be unsatisfiable, while the latter is a fetch failure. Do not use `416` to mean “not cached.” For multiple ranges, use a bounded standards-conforming implementation or an explicit tested fallback; never mislabel one range as satisfying them all. [S6]

A cache miss attaches to a broker demand with a deadline. No scheduler/gateway lock is held while waiting. Before response headers are committed, an unfulfillable fetch can return an appropriate failure. After commitment, a failed stream terminates as incomplete and emits typed out-of-band telemetry; it cannot become a different video or a falsely complete body.

Local validators describe the local representation generation. Rewritten manifests and transformed files get their own metadata. CORS handling and loopback authorization belong to the gateway, but are not substitutes for fetch safety.

## 11.4 Player lifecycle

Use a small measured player pool: normally one active player and at most one warm next player; weak devices may use only one. A larger encoded-byte reserve does not imply more decoders. Release obsolete decoders, MSE buffers, object URLs, and read leases after navigation according to a bounded back-navigation policy.

The player must preserve the poster until the intended presentation is actually ready to replace it, avoid concurrent audible players, and handle autoplay denial as a UI permission state rather than a network outage. Browser `preload` is a hint, not a byte or memory budget. [S15]

Telemetry separates:

```text
swipe/selection committed
player attach / activation requested
startup bytes complete
decoder ready
first frame submitted for presentation
first audio / audio-video gap
buffer interval changes
waiting / stalled / recovery
dropped frames / decode error
pause / seek / loop / background / exit
```

Where available, `requestVideoFrameCallback` is a useful first-frame presentation proxy. It concerns submission to the compositor, not proof of a physical display scanout; background/hidden-player behavior needs backend-specific tests. Never substitute download-complete or `loadedmetadata` for visible first-frame measurement. [S21]

## 11.5 New core interfaces

The following contracts extend, rather than replace, Section 11.2. Implementations may choose different Rust types, but MUST preserve the identities, ownership, and validation boundaries:

```rust
// Digest verification, authority, and media interpretation are independent.
enum MapAuthority {
    Hint,
    PublisherBound { event_id: [u8; 32], primary: [u8; 32] },
    LocallyBound { source: [u8; 32], index_record: u64 },
}
struct MapHandle { digest: [u8; 32], authority: MapAuthority }
struct VerifiedBlock {
    map: MapHandle,
    resource_id: String,
    block_index: u64,
    exact_length: u64,
    payload_digest: [u8; 32],
    provenance_id: u64,
}
enum VirtualPart {
    Generated(std::sync::Arc<[u8]>),
    SourceSlice { binding_id: u64, span: ByteSpan },
}
struct DeferredJob {
    job_id: u64,
    calendar_epoch: u64,
    latest_dispatch: std::time::Instant,
    lease_group: u64,
}
```

Do not expose a public `VerifiedBlock` constructor taking unchecked fields. The verifier alone creates it after validating authority, map/resource membership, exact length, and block digest. All clocks crossing process boundaries require a documented conversion to the receiver's monotonic clock and an epoch; raw `Instant` values are not portable wire timestamps.

| Boundary | Input | Output / failure |
|---|---|---|
| `index_store.lookup` | Source binding, parser/profile version, access context | Reusable exact index or miss; never a URL-only cross-generation hit |
| `readiness_map.accept` | Raw map bytes, externally established binding, limits | Schema/semantic checked map; typed binding/digest/version/limit failure |
| `media_adapter.compile` | Governed source reader, selected tracks, optional map hint | Validated dependency scope / next-input demands / unsupported or malformed result |
| `block_verifier.check` | Bound map resource, block index, staged complete block | Verified-block handle or mismatch; no partial success |
| `virtual_packager.compile` | Locally validated sample index, fixed source bindings, profile/build | Immutable output descriptor and exact lengths, or unsupported/limit failure |
| `virtual_reader.demand` | Descriptor, output span, consumer/epoch/deadline | Generated-byte slices and source demands through the same broker |
| `buffer_model.evaluate` | Current contiguous coverage, schedule, joint service scenarios | Maximum-deficit samples and explicit assumptions; never a byte-count-only readiness flag |
| `calendar.build` | Deduplicated protected DAG, current allocation, service envelope | Joint bounded reservations or infeasible/unknown-for-model |
| `calendar.validate` | Proposed calendar plus current epochs and budgets | Permission to dispatch/defer, or immediate deterministic fallback |

## 11.6 Persistent state and transactions

Use bounded, versioned records for object generations, committed intervals/blocks, source indexes, map bindings, virtual descriptors, and quota reservations. A relational database is suitable but not mandated. Keep blob writes outside the scheduler actor. Publish a readable interval/block only after its bytes and verification/coverage metadata can recover consistently; use temporary staging and an atomic commit protocol supported by the chosen store.

On crash recovery, reconcile retained allocations with metadata and outstanding leases before new work. An ambiguous partially spent network reservation remains conservatively spent until resolved; do not refund bytes merely because the request task disappeared. A descriptor with missing payload remains a descriptor, not ready media. An index's `parser_version` change invalidates interpreted state while preserving independently verified raw payload.

The wire schema, map semantic checks, and pure-function reference vectors in the implementation pack are part of this baseline. The reference functions are deliberately synchronous and network-free; they illustrate cross-language contracts, not production threading, parser isolation, Nostr signature verification, or media decoding. A Rust implementation must port the same observable behavior and add the runtime tests in Appendix B.

# 12. Evaluation and release criteria

## 12.1 Prove the data path before the optimizer

First test generation isolation, exact interval serving, cancellation, quotas, decoder activation, and supported media profiles under controlled faults. Only then compare scheduling policies. Correctness must not depend on a reliable learned model.

Primary experiments compare equal semantic order, supported corpus, cache initial state, cumulative data allowance, storage/RAM limit, and failure conditions. Add rescue reordering as a separate product experiment, not a hidden advantage given only to WARP.

## 12.2 Minimal useful baselines

Use: the current application policy; sequential whole-object preparation; a simple current-plus-next buffer policy; the deterministic WARP core; and optional WARP lookahead. Add targeted ablations for single-flight, structural discovery, local-index retention, cold/cached readiness maps, block authentication/failover, virtual packaging, decoder warming, latest-safe dispatch, deficit-based buffers, conditional resume, and adaptive sizing. This tests implementable causes of improvement rather than maintaining dozens of weak baselines.

Run **client-only** experiments with identical media representations separately from **publishing-profile** experiments that change encoding/layout/replication. The latter include publisher encode cost, total stored bytes across renditions/replicas, startup quality, and watch-weighted quality. Otherwise repackaging or extra storage could be mistaken for a better retrieval algorithm. Include payload-cold/index-warm cases and all-cold cases. Compare virtual packaging against both native passthrough and materialized remux, counting source-scatter requests and transient output memory.

An offline oracle must obey the same resource/semantic/protocol constraints. Perfect future knowledge alone does not make a heuristic oracle a certified upper bound; only call a result a bound when the optimization or relaxation proves it.

## 12.3 Required metrics

| Area | Measurements |
|---|---|
| User-visible | Selection-to-first-frame p50/p95/p99, explicit startup deadline success, startup abandonment/failure, stall time/episodes, first audio and A/V gap, dropped frames |
| Efficiency | Internet payload per watched minute and per successful start, request count, unused prefetched bytes, duplicate bytes, discovery bytes, cache-hit value, cancellation overrun |
| Resources | Peak and integral disk allocation, RAM/decoder memory, scratch storage, CPU/energy where observable, allowance/rate violations |
| Readiness | Actual contiguous current buffer, next activation delay, reserve shortfall, recovery after burst swipes, predicted versus observed miss probabilities |
| Integrity | Generation mixing prevented, block/full-digest distinction, map consistency failures, mismatch provenance, stale bindings |
| Extensions | Sidecar hit rate/bytes/latency, serial discovery stages avoided, authentication overfetch, verified progress retained after failover, virtual header/output memory |
| Deferred service | Bytes avoided on abandonment, missed latest starts, calendar invalidations, conservative-model rejections, actual buffer deficit and sustained-stall outcomes |
| Publication | Actual startup dependency bytes, baseline continuation failures, switch quality/gaps, aggregate rendition/replica storage and encode cost |
| Semantics | Reordering/substitution rate, displacement and deferral age, explicit-intent violations, exposure by origin after conditioning on recommended rank |

Do not report first-frame percentiles only among successful starts without their failure/abandonment denominator. A user who swipes away before a frame arrives is a censored/failed attempt for the relevant metric, not a fast startup. Report warm-cache and cold-cache cases separately and disclose unattributed traffic.

Define usefulness carefully: encoded samples needed for watched playback plus necessary initialization are useful; future bytes retained but not yet reused are not automatically counted as saved traffic. Report actual later reuse separately from predicted cache value.

## 12.4 Experiments and calibration

Cross front/tail metadata, interleaving, keyframe spacing, selected audio, codecs, and manifests with accurate/false/missing metadata, range/validator behavior, abrupt network changes, shared-path contention, origin failures, low storage, and rapid navigation. Include real-device decoder/memory tests, not just HTTP replay.

Recorded throughput depends on the collection policy's timing and concurrency. Shadow mode cannot reveal the outcomes of unexecuted requests or prove a policy's QoE gain. Use controlled network/origin emulation, held-out temporal tests, and randomized canaries for causal conclusions; trace-driven counterfactual limitations are established in CausalSim. [S22]

Bootstrap confidence intervals by session/device/origin where dependence requires it; do not treat correlated chunks as independent users. Publish the corpus/profile coverage and calibration gaps alongside aggregate gains. A tiny sample cannot establish p99 improvement.

Map and virtual-file tests MUST include malicious but validly signed metadata, not just unsigned garbage. A trusted publisher can still make a wrong structure claim. Simulate a fast startup origin with an unavailable continuation origin, and distinguish continuous throughput from bursty segment completion. Compare latest-safe scheduling against identical immediate-fetch jobs under identical service envelopes; then stress envelope violations rather than assuming the conditional result transfers unchanged to the Internet.

## 12.5 Release gates

The deterministic core ships only after all applicable acceptance cases pass, no unaccounted downloader remains, and the target-device matrix is documented. Each enabled extension additionally passes its Appendix B group and integration failures with the core. The network-free reference tests are contract checks, not proof these end-to-end release gates pass. A canary must not materially worsen tail startup, failures, budget compliance, decoder stability, or semantic fidelity at matched budgets. Predeclare the tolerated regression and rollback criteria.

Advanced scheduling remains disabled unless it improves the chosen Pareto trade-off over the deterministic core by more than its CPU, energy, and maintenance cost. A more elaborate model is not itself a release benefit.

# 13. Security, integrity, and privacy

## 13.1 Fetch safety

Treat every event URL and manifest descendant as attacker-controlled. Default to HTTPS; any HTTP compatibility mode is explicit. Reject embedded credentials, unsupported schemes, unexpected ports, and disallowed destinations. Apply policy to parsed hostnames, resolved addresses, redirect destinations, and the actual connected peer, including IPv4-mapped IPv6 and relevant special-address ranges.

Connect only to an approved resolved address while retaining the intended TLS hostname validation. Reapply policy on redirects and connection reuse. Disable environment-proxy surprises and unreviewed alternate-service routing; explicitly configured proxies have their own trust boundary. Do not permit a second unchecked resolution between validation and connection.

Bound headers, redirects, request time, idle time, body reads, manifest expansion, and per-origin fan-out. Strip credentials/cookies/authorization on cross-origin transitions unless an explicit authorized profile says otherwise. Do not forward Nostr private keys, auto-sign origin challenges, or execute authentication embedded in content.

## 13.2 Local service protection

Bind loopback services to intended interfaces only. A random port and CORS alone are insufficient authorization: cross-origin requests can still cause work even when their responses are unreadable.

Use unguessable scoped session/resource capabilities, validated Host/Origin rules where applicable, explicit control-method authorization, and restrictive CORS. The public control surface accepts internal resource IDs, not arbitrary fetch URLs. Native/browser media paths that cannot carry custom headers need a tested scoped capability mechanism. Do not expose tokens in analytics or unrestricted logs.

## 13.3 Parser and transform isolation

The small in-process scanner is allocation-, depth-, count-, and overflow-bounded. Complex demuxing, decoding, thumbnails, and transformations run behind a hardened adapter or sandbox. Workers receive controlled byte input, not network-capable URLs. Restrict filesystem access, subprocess lifetime, memory, CPU, output size, and protocol handlers.

An unknown box or unfamiliar codec is not automatically malicious. Distinguish unsupported feature, corrupt bytes, resource-limit rejection, and parser failure. Fuzz the exact parsers and boundary conversions used in the release.

## 13.4 Privacy and data minimization

Speculative fetches disclose interest and timing to third parties even when the video is never displayed. Bound lookahead and origin fan-out, provide a Data Saver/privacy setting, and stop inappropriate background speculation. A local gateway does not hide the user's IP; a remote gateway changes, rather than removes, the trust boundary.

Keep watch models local by default. Log hashed/partitioned identifiers and error classes rather than full signed URLs, tokens, or raw social content. Bound model/key cardinality and retention; attacker-controlled unique URLs must not create unlimited telemetry state. Respect logout, private-mode, and deletion behavior without pretending removal from one cache erases distributed public copies.

## 13.5 Extension-specific attack boundaries

A publisher-bound map can still be a denial-of-service input. Enforce raw-body, parse-depth, table-cardinality, block-expansion, unit-closure, and map-fetch limits before committing resources. Do not repeatedly walk a large graph per sample on the hot path; cache bounded validated closures, deduplicate shared nodes, and bound compilation work. Map replicas and media descendants cannot bypass SSRF, credential stripping, cache policy, or origin fan-out restrictions.

Only the local packager emits generated container bytes. Remote sample/interval hints must be checked against governed media; no remote descriptor is an executable muxing program. Virtual output uses frozen source bindings and checked arithmetic. A plan-hash collision policy is not a replacement for byte/source validation. On corruption, isolate the implicated map, generation, or attempt instead of poisoning all related content.

The uploader does not publish private URLs, credentials, maps, or replicas without explicit user authorization. A signed URL's token can itself be sensitive. If a resource needs private authentication or expiring access not represented safely by `WRM/1`, use the ordinary authenticated application path rather than embedding secrets in a public sidecar.

# 14. Implementation sequence and limits

## 14.1 Implementation milestones and definition of done

| Stage | Deliverable | Exit evidence |
|---|---|---|
| A — Correct data path | `CORE/3` ownership, typed generations, exact HTTP serving, one broker, quotas, actual player telemetry | Core protocol/broker/security cases; no hidden downloader, generation splice, or false byte completion |
| B — Useful readiness | Selected-track dependency compiler, native/sparse MP4 plus declared HLS/WebM adapters, local index cache, small warm player pool, deterministic current/next policy and deficit model | Real-device media/player cases; correct contiguous coverage, cold/warm separation, ordinary fallback |
| C — Virtual packaging | Bounded `VPK/1` subset, immutable plans, source-span serving, no required second full payload | V-group tests, decode/timeline equivalence on supported corpus, resource comparison against native/materialized paths |
| D — Cooperative fast path | Optional `WRM/1` sidecars, authorized block verification, verified failover, `PUB/1` uploader | R/U-group cases, adversarial signed maps, all-cold map overhead, full-baseline continuation and replica correctness |
| E — Deferred acquisition | Small watch/shared-path models, expiring joint calendars, `JIT/1` under declared envelopes | D/J-group cases, timer/epoch invalidation, matched-service waste reduction, stress failures and calibration |
| F — Measured optional optimization | `LOOKAHEAD/1`, prices, hedges, richer models or materialized transforms | Incremental benefit over the preceding deterministic system at matched constraints; independent rollback |

Stages C, D, and E can be developed independently after their Stage A/B dependencies exist. Disable each extension with a feature flag without bypassing the broker or changing semantic order. Keep the deterministic immediate-protection fallback permanently. Before enabling an extension, implement its failure path as well as its success path.

The release manifest is the implementation's authoritative record of actual backend/profile support and pinned library/protocol snapshots. This document fixes observable behavior; it does not claim that an arbitrary dependency version or device has already passed the test matrix.

## 14.2 Fundamental limits

A slow, nonseekable, tail-initialized file may need complete acquisition before playback. An incompatible codec may require conversion. A rapid burst of swipes can consume any finite reserve; no scheduler can manufacture missing network capacity. An authenticated index removes discovery and preserves verified progress, not unavailable bytes. Virtual packaging removes required materialization, not codec limitations or inaccessible metadata. Startup replicas buy a bridge, not full-video availability. A latest-start schedule is only as valid as its service envelope. WARP must expose these situations without corrupting bytes, exceeding forbidden allowances, or silently changing explicit user intent.

The engineering claim is therefore testable and limited: **under defined devices, budgets, media profiles, and network conditions, WARP should reduce avoidable delay and waste relative to simpler policies.** Its success must be demonstrated with the experiments above, not inferred from the sophistication of its equations.

# Appendix A. Non-normative bootstrap configuration

These are initial experiment settings. Hard security/resource caps are enforced regardless of statistical confidence; optimize the values on the actual corpus and weakest supported devices.

| Setting | Initial experiment setting |
|---|---|
| Candidate metadata window | 8 items; encoded prefetch much narrower |
| Warm players | Active + one next; one total on memory-constrained devices |
| Startup cushion | Seed with 1–2 seconds; evaluate Section 7.5 deficit and immediate needs; insufficient affordable coverage remains degraded |
| Foreground continuation | Approximately 3–6 seconds initially; smaller for Data Saver, larger only when justified |
| Encoded next reserve | Intended next plus at most one additional item initially |
| Initial byte discovery | 32–64 KiB with structural jumps where available |
| Cumulative ordinary discovery | Up to 1 MiB and a small request/time cap, then explicit re-evaluation; not an object-size threshold |
| Body read quantum | At most 64 KiB initially; cancellation independent of quantum completion |
| Global active media requests | Start at 2; allow a third only after shared-path measurements justify it |
| Per-origin media concurrency | Start at 1, permit 2 for independent audio/video or demonstrated latency benefit |
| Same-item alternative attempts | Small bounded sequential fan-out; hedging initially disabled |
| Reorder on delivery speed | Disabled by default |
| Virtual packaging | Off until the exact `VPK/1` backend subset passes; direct playback stays available |
| Readiness maps | Local indexes in core; external `WRM/1` off until authority/validation tests pass |
| Map ceilings | Appendix D; lower device limits allowed, never larger under the same version |
| Block size | 64 KiB publishing experiment; actual size from the bound map; measure amplification |
| JIT payload deferral | Off with unknown service evidence; calendar expiry at most the bounded control horizon |
| Maximum periodic control interval | Start at 100 ms; due deadlines/invalidations trigger earlier work; test on weak devices |
| Publishing startup duration | Start at 1–2 seconds, with full baseline continuation; choose by dependency bytes and quality |
| Transcoding / beam search | Disabled by default |
| Scheduler target | Set a measured low-millisecond p99 target on supported devices; no blocking work in fast path |
| Disk/RAM/data allowance | Explicit product/device/user configuration; never inferred from advertised media size |

Time and byte caps both apply. A minimum-sized transfer that cannot finish before a deadline is not made useful by calling it a small chunk. Low confidence widens margins; it does not relax enforcement.

# Appendix B. Acceptance matrix

The implementation MUST turn applicable cases into protocol, broker, simulator, and real-player tests. The reference pack exercises a subset of pure contracts; it does not execute media/origin/device acceptance. Record pass/fail and profile applicability in the release manifest.

## B.1 Core acceptance cases

| ID | Stimulus | Required result |
|---|---|---|
| H01 | Range `0-65535`, response covers only that prefix | No attempted same-response extension beyond byte 65535; missing bytes require another request |
| H02 | Range request receives `200` | Body starts at local offset zero; continuation requires renewed lease |
| H03 | Nonzero conditional range receives changed full object | New generation; no append to old prefix |
| H04 | Weak ETag only and no independent authentication mechanism | No safe resume claim or cross-request cache assembly under default policy |
| H05 | Different origins send the same ETag | No cross-origin identity inference |
| H06 | Missing/false total length; oversized body | Bounded reads; no complete-object/readiness declaration from size alone |
| H07 | Truncated `206` | Only actual bytes retained; response remains incomplete |
| H08 | Content-coded range despite identity request | No offset mixing with identity representation; bounded fallback |
| H09 | Unknown length and suffix discovery | No mandatory preceding HEAD; validate resulting response |
| H10 | `416`, `429`, `401`, `404` | Distinct bounded recovery; no retry storm or invented credentials |
| I01 | Two endpoints claim same `x` but have no bound block verification | Provisional generations remain isolated |
| I02 | One candidate mirror fails full digest check | Quarantine its generation; preserve known-good copies and unrelated endpoints |
| I03 | Same manifest digest, different relative-URI bases | Independent dependency resolution; no segment integrity inheritance |
| I04 | Transformed/rewritten object | New output digest and generation; original `x` not reused |
| M01 | Large known-size `mdat`, later `moov` | Structural offset jump, not payload download just to locate next box |
| M02 | `moov` string occurs in payload | No false structural match/readiness |
| M03 | Front `moov`, remote first audio sample or delayed random-access point | Startup closure includes real track/sample dependencies |
| M04 | Classic MP4 routed to MSE | Explicit compliant conversion or supported native path; no arbitrary-prefix append |
| M05 | HLS separate audio/init/byte-range segments | All selected dependencies participate in readiness and budgets |
| M06 | Manifest descendant points to a prohibited address | Fetch rejected before connection; parser does not fetch it independently |
| M07 | Unsupported encryption or codec | Typed unsupported-profile outcome; compatible fallback or explicit error |
| P01 | Bytes cached, decoder cold | Cold activation delay retained in readiness/metrics |
| P02 | First frame ready, continuation missing | Not counted as sustainable reserve coverage |
| P03 | Buffer has a gap or audio lags video | Coverage ends at first required-track gap |
| P04 | Rapid A→B→C swipes with late A/B callbacks | C's epoch/state never overwritten |
| P05 | Autoplay denied or app backgrounded | Permission/lifecycle handling; no origin-failure learning |
| P06 | Backward swipe or explicit seek | Intent preserved; correct random-access plan; no unrelated rescue substitution |
| B01 | Player, parser, prefetch request overlapping intervals | One missing-byte union, reference-counted consumers, no double byte charge |
| B02 | One consumer cancels while another needs bytes | Shared request survives at remaining demand's priority |
| B03 | Slow body stops progressing | Timer/cancellation interrupts without waiting for full progress block |
| B04 | Rate bucket refills after cumulative allowance exhausted | No new Internet bytes admitted merely due to rate refill |
| B05 | Current rescue, an open-ended player request, and background work compete | Urgent bounded dependency path receives service; later bytes are not all emergency work |
| B06 | Storage full, crash, or transform output expansion | No unreserved growth; recover only committed intervals; all leases released |
| B07 | Huge logical sparse length or many tiny intervals | Bounded metadata/allocations; no zero-hole serving |
| B08 | Cache says `no-store` or user logs out | No unauthorized reusable persistence/access |
| S01 | Two origins share device bottleneck | Joint predicted throughput is allocation-limited, not sum of isolated rates |
| S02 | Watch time already elapsed | Remaining distribution is conditioned on survival |
| S03 | Unreachable future unit | No unconditional startup/coverage reward |
| S04 | Multiple delayed dependent units | Actual waiting counted once via playback simulation |
| S05 | Same rollout with auxiliary scores disabled | No duplicated resource/tail/rank/cache terms |
| S06 | Planner times out or uses stale snapshot | Deterministic fallback; stale proposal revalidated or discarded |
| X01 | Redirect/DNS rebind to loopback/private destination | No disallowed peer connection |
| X02 | Hostile website reaches local gateway | No unauthorized URL fetch/control, even if CORS hides response |
| X03 | Parser input causes huge allocation/nesting | Bounded rejection/worker termination; scheduler and playback stay responsive |


## B.2 Extension and integrated-controller acceptance cases

| ID | Stimulus | Required result |
|---|---|---|
| R01 | Local index survives payload eviction, then a changed origin generation appears | Immutable facts remain stored but cannot authorize reads against the changed generation |
| R02 | Map JSON is reformatted without changing its parsed values | Raw-body digest changes; old commitment does not authenticate new bytes |
| R03 | A third party supplies matching block hashes and its own signature | Map remains a hint absent publisher/local authority; no authenticated cross-mirror assembly |
| R04 | Seven startup blocks verify; source stalls on the eighth | Eligible alternate supplies only missing authentication units; retain verified siblings |
| R05 | Final block is shorter than block size; sender pads it | Exact unpadded final length required; padded data does not verify |
| R06 | One block's bytes or index are wrong | Reject that block/attempt; no shared authenticated commit or premature readiness |
| R07 | All blocks match but full resource digest does not | Map consistency failure; no false whole-file success or blanket origin ban |
| R08 | Map unavailable, over limit, or unsupported version | Ordinary retrieval remains eligible; no mandatory map wait loop |
| R09 | Duplicate JSON fields, unknown fields, cyclic/missing dependencies | Bounded typed map rejection; no parser ambiguity, recursion loop, or hidden code execution |
| R10 | Offsets exceed `2^53` but fit u64, or exceed u64 | First remains exact; second rejected; no JSON-float rounding |
| R11 | Map describes only initial intervals or omits real audio/preroll needs | No implied full timeline; actual adapter expands/invalidates closure before readiness |
| R12 | Cold sidecar costs more than direct startup acquisition | Planner may bypass it; map benefit is not unconditional |
| R13 | Addressable event updates its map while playback is active | New selection uses new binding; pinned old epoch is not mutated in place |
| R14 | Source/index has private or `no-store` policy; user logs out | No reusable unauthorized index/payload access; release relevant capabilities |
| R15 | A 5 KiB sample intersects a 64 KiB authenticated block | Lease and deadline include complete authentication-unit acquisition |
| R16 | Authorized map digest matches, but primary binding does not | Reject that binding; no association with unrelated `imeta` |
| R17 | Same primary has conflicting map digests in one event revision | Disable that extension binding; ordinary post playback remains |
| R18 | Bound manifest has uncommitted descendant media | Manifest verified alone; descendants retain their own verification state |
| V01 | Virtual range crosses generated headers and multiple source slices | Exact output bytes, offsets, lengths, and deduplicated source demands |
| V02 | Source fails after virtual response headers commit | Incomplete termination and typed telemetry; never zero-fill or report successful complete body |
| V03 | Virtual descriptor cached but source samples evicted | Descriptor is reusable knowledge, not `StartupBytesReady` |
| V04 | Plan digest is known before output body is hashed | Plan/local validator never advertised as original or output body SHA-256 |
| V05 | Source generation changes during virtual read | Existing view fails/invalidate; replacement requires new view/epoch |
| V06 | B frames, composition offsets, field-width boundaries | Correct supported timestamps/fields or explicit profile rejection; no silent timestamp flattening |
| V07 | Edit lists, open-GOP recovery, changing descriptions, or external references outside profile | Typed unsupported result and ordinary alternative; no invented compatibility |
| V08 | Remote map attempts to inject generated bytes or a muxing command | Schema rejection; only local validated compiler produces virtual headers |
| V09 | Large source and many virtual slices under memory pressure | Bounded slice metadata/read buffers/leases; no automatic whole-output materialization |
| V10 | Nice virtual output requires highly scattered origin reads | Compare actual source request/overfetch cost; do not assume virtual layout fixes origin layout |
| D01 | Arrival happens at the exact buffer-empty deadline | Evaluate left limit; no positive-duration stall if usable bytes arrive exactly on time |
| D02 | Several arrivals have equal timestamps | Count pre-arrival deficit once and sum only newly usable coverage |
| D03 | Audio delayed, video far ahead | Deficit uses required-track contiguous coverage, not video-only arrivals |
| D04 | Continuous service yields 0.8 media seconds/second for a 20-second horizon | Four-second initial deficit under that model; no confusion with burst arrivals |
| D05 | 0.8-second batches arrive only at each integer second | Twenty-second maximum pre-arrival deficit is 4.8 seconds, not the continuous-model four seconds |
| D06 | Required buffer exceeds byte/RAM/data allowance | Select admissible alternative or degraded readiness; never silently label a capped buffer safe |
| D07 | Initial buffer/rendition changes the remaining sample set | Recompute acquisition/service trajectory; do not reuse inconsistent arrivals |
| D08 | Model quantile is based on few/correlated scenarios | Report model/estimation limits; no implied Internet reliability guarantee |
| D09 | Playback rate doubles, app pauses, or transport stalls | Consumption follows commanded playback; transport stalls do not improve predicted feasibility |
| J01 | Two 700 ms jobs share a 2 s deadline on a serial lane | Latest reservations occupy 600–1300 ms and 1300–2000 ms, not two overlapping 1300–2000 ms slots |
| J02 | Future bandwidth/epoch changes after deferral | Invalidate calendar, wake controller, run bounded immediate protection |
| J03 | User leaves before a safely deferred job starts | Drop consumer and release unused reservations; no unnecessary payload request |
| J04 | Early transfer opportunity will disappear | Time-varying envelope prevents unsupported deferral or JIT is disabled |
| J05 | Calendar uses decoder/input deadline earlier than buffer depletion | Earlier actual dependency deadline wins |
| J06 | Player, parser, and verifier share dependencies | One deduplicated job and resource reservation, with earliest downstream deadline |
| J07 | A fixed serial order cannot fit but a parallel schedule might | Report infeasible-for-model, not proof no recovery exists; evaluate allowed alternatives |
| J08 | Latest-start timer, expiry, or periodic wake fires | No indefinite sleep; revalidate epochs/leases before dispatch or deferral |
| J09 | Origin stream cannot be paused/resumed cheaply | Include active work and setup/cancellation cost; no zero-cost preemption assumption |
| J10 | Insufficient service evidence | Aggressive deferral disabled; fixed conservative margins plus unknown-risk reporting |
| J11 | Background job fits time slack but consumes protected data/disk quota | Admission rejected; deferred job's bounded protected resource reservation remains |
| U01 | High-quality upgrade is late | Baseline continues at supported quality; no artificial wait for improvement |
| U02 | Publication contains only a short teaser and a separate main video | Teaser cannot satisfy the full-baseline or seamless-continuation contract |
| U03 | Replicated startup is available but continuation is not viable | Report limited/degraded readiness; do not claim sustainable playback |
| U04 | Renditions share event/`ox` but not aligned timing | No seamless switch claim; stay on rendition or controlled restart |
| U05 | Publisher omits all WARP tags/sidecars | Ordinary Nostr media path still works within backend capabilities |
| U06 | Replica hosts startup resources but not the rest | Endpoint discovery does not assume full-video presence; explicit continuation path used |
| U07 | Upgrade succeeds and old rendition is no longer needed | Release obsolete demands; no unbounded duplicate-rendition fetching |

# Appendix C. Worked decisions

**Tiny object, useful response reuse.** A 350 KiB object has credible size evidence and an already-open low-latency path. The planner may select one bounded full GET. If it selected a 64 KiB range instead and received only that range in a `206`, it needs another request for the remaining 286 KiB. If the server ignored Range and returned `200`, it may continue that response after renewing its lease. These are different actions and different request costs.

**Tail metadata without blind tail expansion.** A 24 MiB MP4 begins with a valid `ftyp` and a sized `mdat`. The iterator requests the next box header at its calculated offset, finds `moov`, and asks the adapter for actual startup audio/video ranges. If version evidence permits sparse assembly, it avoids downloading unrelated payload. If not, it keeps a continuous-response/whole-object alternative instead of pretending the sparse bytes are safely versioned.

**Cached is not immediately visible.** The intended next item has all startup bytes cached but a cold decoder; a different item is warm. With display reordering disabled, WARP activates the intended item and records its real first-frame delay. It does not label a substituted warm video as a transport optimization with unchanged semantics. An enabled rescue policy may make a different choice, but must report displacement and never apply it to an explicitly opened post.

**Correlated reserve failure.** Two future videos have small startup closures on different hostnames, but the device connection degrades. The planner reduces both completion estimates under the shared-path model, protects active continuation, and suspends extra preparation. It does not multiply two optimistic independent readiness probabilities into a false reserve guarantee.

**Authenticated progress across origins.** A bound resource has eight required startup blocks. Seven verify before origin A stalls. Origin B supplies the missing block; WARP verifies it before joining the same authenticated resource view. A same-`x` URL without the bound block table would not authorize that splice. If all blocks later match but the complete-file digest fails, the map/resource binding is inconsistent and must not be reported as whole-file verified.

**Virtual output is a view, not another required stored video.** A fragment contains generated headers followed by source slices `[1000,9000)` and `[12000,16000)`. An output request crossing a header/sample boundary maps to the corresponding generated slice and missing source intervals. Output length is fixed; source reads share the broker. Descriptor existence alone does not mark the fragment ready, and a 3,000-byte source gap is neither downloaded nor zero-filled unless an admitted coalescing request actually includes it.

**Right-justified shared service.** Two independent 700 ms jobs each have a two-second deadline on one serial service lane. Their latest shared schedule is 600–1300 ms and 1300–2000 ms. Scheduling both at 1300 ms from their individual slack calculations double-books the lane. A service-epoch change invalidates the calendar and wakes the controller.

**Average rate versus useful arrivals.** Continuous arrivals of 0.8 media seconds per second create a four-second maximum deficit over 20 seconds at normal playback. If instead 0.8-second segments become usable only at each integer second, the pre-arrival deficit at 20 seconds is 4.8 seconds. The reference tests cover the latter. Same mean throughput does not imply the same initial buffer requirement.

# Appendix D. `WRM/1` wire and semantic contract

This appendix is normative for the optional `WRM/1` extension. It is a WARP application format, **not an assigned NIP, registered media type, or replacement for NIP-71/94**. Existing event fields keep their existing meanings. No client is required to support this extension to play ordinary media.

## D.1 Publication and commitment

In a correctly verified Nostr video event, the proposed WARP tag has exactly five strings:

```text
["warp", "wrm/1", "<primary-file-sha256>", "<map-body-sha256>", "<absolute-https-map-url>"]
```

The primary digest MUST match the `x` of the exact event `imeta` representation being described. Verify the normal Nostr event ID/signature using an established Nostr implementation before accepting this binding. The event's publisher authorizes the map for that representation. Nostr's tag/event structure permits application metadata, but the meaning of this new tag is defined only by WARP. [S25]

For one primary digest in one event revision, accept one map digest with up to four replica URLs. Conflicting map digests for that primary invalidate the extension binding for that primary; use ordinary playback. Exact duplicates deduplicate. Limit acquisition to admitted sequential candidates or explicitly budgeted races. A tag on an unrelated publisher's event is a hint unless a separate explicitly supported authorization profile proves delegation; `WRM/1` itself defines no delegation.

The map is UTF-8 JSON served as `application/json` or an ordinary binary body, with identity content coding. Hash the exact received representation body bytes, excluding HTTP transfer framing. Do not reserialize JSON before comparison. Whitespace/key order changes produce a different map digest. The producer finishes final bytes before computing the digest and signing the event. The map contains no self-digest or event-ID backreference.

The expected map and primary digests MUST come from a verified event commitment or an established local record; receiving those digests next to an untrusted map is not authentication. A local compiled index is not required to use this wire format, but local authenticated block commitments must be derived from actually verified complete bytes or another authorized binding, not a filename claim.

## D.2 Encoding and global bounds

The map root has exactly `format`, `primary`, `resources`, and `presentations`. `format` equals `wrm/1`. Reject unknown fields in this version; add future incompatible fields under a new version. Reject invalid UTF-8, BOM, duplicate object keys, non-finite numbers, unpaired Unicode surrogates, extra trailing data, and nesting beyond 32 containers.

All byte offsets, lengths, and microsecond timestamps are canonical unsigned decimal **strings**: `0` or a nonzero digit followed by digits, bounded by `2^64-1`. This avoids JSON/JavaScript floating-point offset truncation. Bounds comparisons and arithmetic are checked integers. The block size is a small JSON integer from the explicit set below, encoded without a fractional or exponent part; booleans are not integers. Hashes are 64 lowercase hexadecimal characters. Identifiers match `[A-Za-z0-9_-]{1,64}`.

| Limit | `WRM/1` maximum |
|---|---|
| Raw map body | 2,097,152 bytes |
| JSON nesting | 32 |
| Resources | 4,096 |
| Presentations | 8 |
| Tracks per presentation | 2; exactly one video and at most one audio |
| Units across the map | 8,192 |
| Source spans across initialization and units | 32,768 |
| Block hashes across all resources | 16,384 |
| URLs per resource / replicas per tag binding | 4 |
| URL length | 4,096 UTF-8 bytes |
| Unit dependency references across the map | 32,768 |

These are parser/admission ceilings, not targets or prefetch permissions. A device may enforce lower limits and report `UnsupportedProfile`/`MapLimit`, then use ordinary discovery. The JSON Schema describes syntax and local bounds; the cross-field, aggregate, trust, URL-policy, and media checks below are also mandatory. Schema success alone is insufficient.

## D.3 Resource records

Each resource contains exactly the required fields below and optional `blocks`:

| Field | Type | Meaning |
|---|---|---|
| `id` | identifier | Unique within `resources` |
| `sha256` | lowercase digest | Expected digest of the entire identity-coded resource body |
| `length` | positive u64 decimal string | Exact declared byte length; verified separately against bytes |
| `media_type` | string, 1–128 characters | MIME hint; never a parser/codec guarantee |
| `urls` | 1–4 absolute HTTPS strings | Candidate locations of this exact complete resource; no relative resolution, embedded credentials, fragments, raw whitespace/control characters, or backslashes |
| `blocks` | optional object | Exactly `size` and `sha256`; defines the complete block partition |

`blocks.size` is one of `16384, 32768, 65536, 131072, 262144, 524288, 1048576`. `blocks.sha256` has exactly `ceil(length/size)` digest entries, ordered by block index. Entry `j` hashes the raw bytes `[j*size, min((j+1)*size,length))` with SHA-256, without prefix, suffix, index encoding, or padding. Context comes from the bound map, resource ID/length, and fixed position; this is not a Merkle tree.

Repeated resource digests with different lengths are a consistency error. Every span references an existing resource ID and satisfies `0 <= start < end <= length`. A whole-file hash is checked only after exactly `length` body bytes exist; shorter prefixes or zero-filled holes do not qualify. All resource URLs pass the ordinary fetch gate, including redirects and actual destination verification. The map cannot authorize otherwise forbidden network access.

The `primary` root field names an existing resource ID. Its digest must equal the commitment's primary digest. For a manifest-based presentation the primary may be the original playlist; referenced media have their own resource records. Rewriting a playlist gives a new local identity and does not change the committed original resource.

## D.4 Presentation and unit records

Each presentation contains exactly these fields:

| Field | Type | Meaning |
|---|---|---|
| `id` | identifier | Unique within the map |
| `profile` | string enum | `native-mp4/1`, `mse-fmp4/1`, or `hls-vod/1`; an intended adapter path, not proof of backend support |
| `duration_us` | positive u64 decimal string | Declared presentation duration |
| `tracks` | track array | Required selected tracks, each `{id, kind, codec}`; exactly one video, at most one audio; IDs unique locally |
| `init` | nonempty span array | Shared initialization/structure dependencies for this presentation |
| `units` | nonempty unit array | Proposed useful playback intervals and dependency relationships |

A track's `kind` is `video` or `audio`. `codec` is a bounded 1–128-character hint; actual decoder configuration comes from validated media. Different audio selections use different presentation records. This first sidecar profile does not define multi-angle video, mandatory subtitle rendering, DRM, or live timeline updates.

A span is exactly `{resource, start, end}` with half-open byte offsets. A unit is exactly:

```text
{id, kind, start_us, end_us, independent, requires, spans}
```

`kind` is `startup`, `continuation`, or `seek`. `requires` is a list of other unit IDs in the same presentation. Unit IDs are unique; duplicate dependency edges, missing references, self-dependencies, and cycles are invalid. Each unit contains one or more spans and satisfies `0 <= start_us < end_us <= duration_us`. It claims usable coverage for **all required tracks** after its dependency closure and initialization are satisfied. The adapter—not the claim—establishes actual coverage.

`startup` units start at zero. `startup` and `seek` units must have `independent=true` and an empty `requires` list. An independent unit declares that its own spans plus `init` include every preroll/random-access requirement; that declaration still requires adapter validation. A non-independent continuation must have at least one prerequisite. `independent=true` always requires an empty prerequisite list. Each presentation must include at least one startup unit. Overlapping intervals and partial timeline descriptions are allowed; they cannot be added as though they were disjoint or complete.

The transitive byte closure for unit `u` is the union of `presentation.init`, `u.spans`, and the spans of recursively required units. Union by resource identity and verified generation; do not add overlapping spans twice. Authentication expands this closure to complete required blocks or full resources where necessary. A sparse dependency map does not guarantee that a native player will never request additional bytes; those demands enter the broker and invalidate overly optimistic readiness predictions.

Microsecond intervals are planning descriptions. Parsers retain exact source ticks/time bases and perform conservative boundary validation. Do not generate codec timestamps or silently fill a sub-microsecond media gap from rounded sidecar times. Changing a selected track, source generation, or interpretation profile invalidates the corresponding validated scope.

## D.5 Receiver procedure

```text
read an already eligible, correctly verified event revision
select the exact imeta representation and its expected primary digest
resolve one unambiguous WARP map binding, or use ordinary discovery
acquire bounded raw map bytes through the broker
check exact map-body SHA-256 against the authorized binding
parse bounded JSON; reject ambiguous/unsupported syntax
validate local and aggregate limits, IDs, spans, block counts and acyclic dependencies
check primary resource digest against the event binding
build candidate closures; run ordinary security/accounting gates on every URL
fetch and authenticate required resource/block units under leases
validate media structure and required track coverage through the actual adapter
prepare the player and issue readiness only at the achieved evidence level
```

A failure at any map-specific step disables that shortcut with a typed reason; it does not erase valid source bytes or remove ordinary retrieval plans. A passed schema or block hash is never substituted for a successful media parser/backend check.

## D.6 Deliberate exclusions

`WRM/1` has no recursive maps, proof requests, arbitrary signature schemes, URL rewriting programs, compressed-map transport profile, inline binary initialization, remote mux recipes, or detached-authority discovery. These may be designed as explicit later versions only after the small profile demonstrates value. Flat-map overhead and authentication overfetch are mandatory evaluation metrics.

# Appendix E. Executable reference contracts and validation boundary

The accompanying `warp-v3-implementation` pack contains the authoritative matching `schemas/wrm-v1.schema.json`, network-free reference functions in `reference/warp_contracts.py`, synthetic inputs under `examples/`, and `unittest` cases under `tests/`. The schema is JSON Schema draft 2020-12; runtime semantic/trust/media checks in Appendix D still apply. [S27]

Run from the unpacked directory with Python 3.10 or newer:

```sh
python -m unittest discover -s tests -v
```

The suite checks strict map parsing and limits, primary/body binding consistency, exact block lengths/digests, distinction between block and complete-resource verification, dependency closure and authentication amplification, half-open HTTP ranges, virtual cross-part reads, buffer-deficit left limits, and the fixed-order serial calendar. Deterministic randomized checks compare interval unions and deficit calculations to small discrete references; a small exhaustive search checks serial feasibility.

The fixture payload is deliberately **not a playable video**. Its map is a synthetic contract vector; the declared media timeline is not established by these tests. The example tag contains computed digests but no actual verified Nostr signature. A successful fixture test therefore cannot produce a production `AdapterValidated`, `DecoderReady`, or publisher-authenticated state by itself.

The reference module performs no network requests, Nostr signature validation, media parsing, muxing, playback, persistent transactions, or transport accounting. Those remain implementation obligations tested by Appendix B. The included validation report records what was actually executed for this document revision. It is not a benchmark report or a completed release-conformance certificate.

# Appendix F. Consolidated decisions from v2 to v3

| Decision | Final implementation consequence |
|---|---|
| Keep retrieval/readiness/semantic separation | No recommender replacement; no faster-host-driven display reordering by default |
| Promote cheap virtual packaging, not routine transcoding | `VPK/1` is a bounded locally compiled rung; full remux/transcode remains separately admitted |
| Retain structural knowledge independently | Local indexes are core; maps/descriptors do not pin or imply ready payload |
| Make authentication useful before complete download | Optional bound block/resource checks preserve verified progress across endpoints |
| Avoid unnecessary new wire complexity | `WRM/1` uses a capped flat block list; trees, proofs, recursive maps, and remote recipes are deferred |
| Make favorable publication optional | `PUB/1` supplies a full baseline, small startup closure, and explicit continuation/replica paths |
| Treat deliberate waiting as a real action | `JIT/1` requires a joint expiring calendar and protected quotas, not individual throughput guesses |
| Protect the continuation trajectory | Deficit-based buffer tests include burst arrivals, required tracks, and local processing |
| Preserve correctness despite model error | Hard leases, epochs, generation isolation, input bounds, fallback and honest degradation remain mandatory |
| Separate design from evidence | Pure-function tests are executed separately from media/device conformance and performance evaluation |

# References and source boundary

The supplied August 2026 manuscript and WARP v2 are the design lineage. The references below support specific protocol, platform, or research statements. The newly specified profiles, contracts, thresholds, scheduling policy, and acceptance rules are WARP design choices; none is presented as an already adopted Nostr standard or an inherited optimality result. Mutable documentation was checked on 4 September 2026; release builds must pin exact protocol snapshots and dependency versions. No benchmark gains are asserted for this revision.

- **[S1]** Nostr NIP-71, Video Events: `https://github.com/nostr-protocol/nips/blob/master/71.md`
- **[S2]** Nostr NIP-92, Media Attachments Metadata: `https://github.com/nostr-protocol/nips/blob/master/92.md`
- **[S3]** Nostr NIP-94, File Metadata: `https://github.com/nostr-protocol/nips/blob/master/94.md`
- **[S4]** Nostr NIP-B7, Blossom Media: `https://github.com/nostr-protocol/nips/blob/master/B7.md`
- **[S5]** Blossom BUD-01 and BUD-03, blob retrieval and server discovery: `https://github.com/hzrd149/blossom/blob/master/buds/01.md` and `https://github.com/hzrd149/blossom/blob/master/buds/03.md`
- **[S6]** IETF RFC 9110, HTTP Semantics, especially §§8.8, 9.3.2, 13.1.5, 14, and 15.3.7: `https://www.rfc-editor.org/rfc/rfc9110.html`
- **[S7]** IETF RFC 9111, HTTP Caching, especially §§3.3–3.5, 4.1, and 5.2: `https://www.rfc-editor.org/rfc/rfc9111.html`
- **[S8]** WHATWG Fetch Standard: `https://fetch.spec.whatwg.org/`
- **[S9]** IETF RFC 8216, HTTP Live Streaming, especially §§3, 4.3, and 6.3: `https://www.rfc-editor.org/rfc/rfc8216.html` — used for the baseline VOD profile, not a claim of support for every later HLS extension.
- **[S10]** W3C ISO BMFF Byte Stream Format: `https://www.w3.org/TR/mse-byte-stream-format-isobmff/`
- **[S11]** GPAC MP4Box.js documentation, progressive parsing, next-offset input, and segmentation: `https://github.com/gpac/mp4box.js/`
- **[S12]** W3C WebM Byte Stream Format: `https://w3c.github.io/mse-byte-stream-format-webm/`
- **[S13]** W3C Media Capabilities: `https://www.w3.org/TR/media-capabilities/`
- **[S14]** FFmpeg Formats, MOV/MP4 fragmentation and `faststart`: `https://ffmpeg.org/ffmpeg-formats.html`
- **[S15]** WHATWG HTML, media elements and preload/playback lifecycle: `https://html.spec.whatwg.org/multipage/media.html`
- **[S16]** reqwest ClientBuilder documentation: `https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html`
- **[S17]** Chrome Local Network Access documentation and platform feature: `https://developer.chrome.com/blog/local-network-access` and `https://chromestatus.com/feature/5152728072060928`
- **[S18]** hls.js API and DASH-IF dash.js ABR configuration: `https://github.com/video-dev/hls.js/blob/master/docs/API.md` and `https://dashif.org/dash.js/pages/usage/abr/settings.html`
- **[S19]** Li et al., Dashlet: Taming Swipe Uncertainty for Robust Short Video Streaming, USENIX NSDI 2023: `https://www.usenix.org/conference/nsdi23/presentation/li-zhuqi`
- **[S20]** Li et al., TLadder: QoE-Centric Video Ladder Optimization with Playback Feedback at Billion Scale, ACM SIGCOMM 2025, DOI `10.1145/3718958.3750500`. Public author abstract: `https://conferences.sigcomm.org/sigcomm/2025/program/papers-info/` — only the high-level inspiration is used here.
- **[S21]** HTMLVideoElement.requestVideoFrameCallback specification: `https://wicg.github.io/video-rvfc/`
- **[S22]** Alomar et al., CausalSim: A Causal Framework for Unbiased Trace-Driven Simulation, USENIX NSDI 2023: `https://www.usenix.org/conference/nsdi23/presentation/alomar`

- **[S23]** Mike Starr / Meta, “Virtual Video Files at Scale: Seamlessly Processing Billions of Videos Per Day,” @Scale, 29 November 2023: `https://atscaleconference.com/virtual-video-files-at-scale-seamlessly-processing-billions-of-videos-per-day/` — precedent for lazy generated metadata and source-sample views, not a WARP latency measurement.
- **[S24]** BitTorrent BEP 52, The BitTorrent Protocol Specification v2: `https://www.bittorrent.org/beps/bep_0052.html` — authenticated block-tree precedent only; `WRM/1` uses a different bounded flat-list contract.
- **[S25]** Nostr NIP-01, Basic protocol flow description, event/tag/signature structure: `https://github.com/nostr-protocol/nips/blob/master/01.md`
- **[S26]** S. Sen, J. Rexford, D. Towsley, “Proxy Prefix Caching for Multimedia Streams,” IEEE INFOCOM 1999, DOI `10.1109/INFCOM.1999.752149`; author-university record: `https://collaborate.princeton.edu/en/publications/proxy-prefix-caching-for-multimedia-streams-2/` — prefix caching precedent, not a WARP replication result.
- **[S27]** JSON Schema, draft 2020-12: `https://json-schema.org/draft/2020-12` — schema dialect; additional WARP semantic, trust and media checks are specified in Appendix D.
