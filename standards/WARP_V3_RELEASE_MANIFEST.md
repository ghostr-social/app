# WARP v3 native release manifest

Status: native CORE/3 local integration validated; no production deployment performed.
Baseline: WARP-v3-final.md revision 3.0, 4 September 2026.
Implementation: 39c00cd4; local merge: 18624a78; host-test cleanup: 3cfab223.
Raw device runs record source fingerprints, with explicit post-run attestations.

## Profiles and platform boundary

CORE/3 is enabled. WRM/1, VPK/1, PUB/1, JIT/1, and LOOKAHEAD/1 are disabled.
External sidecars are not required or treated as authenticated source bytes.
The referenced implementation pack is unavailable; no reference-pack pass is claimed.

The physical validation target is Xiaomi M2012K11AG, Android 13, arm64,
serial 22e0d933, using the vendored Android video_player backend and Media3 1.9.2.
The physical acceptance corpus covers clear finite H.264 MP4, native progressive
playback, and the supported HLS VOD adapter. Required audio-track timing and
coverage are checked with synthetic parser/dependency tests. Native demux/decoder configuration
remains authoritative; MIME or advertised hashes do not certify codec support.
MP4 edits, unsupported sample descriptions/track topology, unsupported HLS
features, and decoder failures retain bounded ordinary or compatible-rendition
fallbacks. DASH, RTSP, live, DRM, arbitrary encryption, virtual packaging,
browser-only playback, and iOS physical conformance are not claimed.

## Pinned snapshots

| Component | Snapshot |
|---|---|
| Flutter / Dart | 3.47.0 / 3.13.0 stable; framework 4cf24164269a5ebf0c16a028a00727d0e77bbb05 |
| flutter_rust_bridge | 2.7.0 |
| video_player | 2.13.0 |
| Vendored video_player_android | 2.12.0, repository source included in build |
| Vendored video_player_avfoundation | 2.11.0; no physical iOS claim |
| Android Media3 | 1.9.2 |
| reqwest / rustls | 0.12.28 / 0.23.43 |
| tokio / axum | 1.53.1 / 0.8.9 |
| serde_json / sha2 | 1.0.151 / 0.10.9 |
| Local compiled index | format 1, parser mp4-v4, all-supported-tracks/native-progressive-v1 |

Cargo.lock SHA-256: `a6c3bc5f36a790e0fce51a171f925cf35048965ac64dfac424fa7905ea096366`.
Pubspec.lock SHA-256: `5549821fa6aa3a2c2b286e1fec9d3126496de3e5a6538224be578039915880f0`.
Specification SHA-256: `03c61b9de78dd7a73892f2f4553ea974f34ce720f645bbc8065a594a54da6575`.

The pinned reqwest feature closure is rustls-tls plus stream; HTTP/2, HTTP/3,
automatic decompression, proxy discovery, and cookie storage are not enabled.
Default reqwest protocol-NACK replay branches require HTTP/2 or HTTP/3; they are
absent in this build. Redirects are broker-owned and re-admitted per hop.
This native compatibility build explicitly supports public HTTPS on port 443
and legacy public HTTP on port 80; other ports are rejected. The debug/device
fixture profile separately allows only its configured local test origin.

## Resource and integrity policy

| Resource | Enforced limit / declared behavior |
|---|---|
| Metadata / encoded future window | Eight / two items ahead |
| Native player pool | Active + immediate next; one total under memory pressure |
| Requests | Two global, one per origin; current-playback reservations take precedence |
| Transport body reservation | Planned envelope plus 512 KiB bounded headroom; actual response contract still enforced |
| Blocked-read continuation slice | At most 256 KiB, clipped to actual source extent |
| MP4 parser input / allocation | 8 MiB / 8 MiB |
| MP4 structural box / nesting / boxes | 4 MiB / 8 / 4,096 |
| MP4 tracks / samples / table work | 16 / 200,000 / 800,000 |
| Local indexes | 2 MiB per record; 128 records / 16 MiB, inside shared disk budget |
| Indexed service calculation | At most 8,192 samples; 20 s wall horizon; at most 120 steps |
| General deficit calculator | 120 s maximum horizon; 4,096 arrivals |
| Buffer target | 4 s seed minimum, clipped at media end; 30 s steady limit; uncapped requirement retained |
| Volatile progressive media | 16 MiB/object, 32 MiB pool, 32 objects; leased/reserved buffers protected |
| HLS encoded cache | 32 MiB RAM |
| Response headers | 32 KiB aggregate, 128 headers |
| Redirects | At most ten; each hop independently guarded and admitted |
| Default unknown-body envelope | 8 MiB before explicit renewed policy |
| Cumulative Internet allowance | Unlimited by default; configured finite limits survive restart and do not refill |
| Disk storage | Explicit product/runtime capacity; indexes share it with payload |

Provisional sparse bytes require current same-source strong-validator authority.
Whole-file digest success does not grant arbitrary future cross-origin range
assembly. Transformed output has a separate identity. Derived index lookup needs
fresh source validation and matching representation, total, parser, and backend
profile. A cache hit restores structure, not startup bytes or decoder readiness.

The media root is a single-device public cache, not a multi-user server cache.
Only unambiguous public responses permit durable derived-index reuse. Signed URLs,
explicit private/no-store/no-cache, Vary, and Set-Cookie use volatile progressive
buffers. Validatorless responses without explicit public policy also use volatile
buffers. Other partitioned responses cannot authorize durable derived-index reuse. Logout revokes progressive capabilities and HLS
sessions and removes private bytes; retained public media needs fresh authorization.

## Acceptance evidence

Flutter analysis and all 1,890 Flutter tests pass. Line coverage is 98.80%, with
all 493 per-file gates and executable-source representation passing. The complete
physical matrix passes 26 cases, followed by two offline restart phases and a
real HOME/foreground case. The normal ARM64 app is installed and cold-launched
successfully; the welcome screen was visually checked, with no account configured.
Axiom passes all 3,409 Rust files, including Clippy, rustdoc, and semantic policies.
`make native-test` passes all 2,375 tests across 450 target results. The landing
page dry-run validation also passes. The [audit](WARP_V3_IMPLEMENTATION_STATUS.md)
records source attestation and command results. No measured optimization gain or
Internet-service reliability guarantee is claimed.
