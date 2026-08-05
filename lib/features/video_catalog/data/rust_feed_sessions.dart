import 'dart:developer';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_session.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// How many Rust feeds one source holds open at a time. Reopening a
/// feed costs a cold relay round trip, so feeds stay; but each one
/// keeps one snapshot stream, so a source holds only what a viewer
/// moves between: the feed they are in, the one they came from, and
/// the search or profile they just opened. Canonical feed rows are
/// bounded natively; active query feeds preserve their discovered history.
const rustFeedLiveLimit = 3;

/// The Rust feeds one source holds open, keyed by the spec that named
/// them and capped at [rustFeedLiveLimit]: the least recently pulled
/// feed closes when a new one opens past the cap.
final class RustFeedSessions {
  RustFeedSessions({required RustFeedPort port, required Duration deadline})
      : _port = port,
        _deadline = deadline;

  final RustFeedPort _port;
  final Duration _deadline;

  /// Insertion order is recency: the least recently pulled feed first.
  final Map<RustFeedSpecKey, Future<RustFeedSession>> _live =
      <RustFeedSpecKey, Future<RustFeedSession>>{};
  final Map<RustFeedSession, int> _pinCounts = <RustFeedSession, int>{};
  RustFeedAccountSession? _nativeSession;
  int _generation = 0;
  Future<void> _adoption = Future<void>.value();

  /// The live feed for [spec], opening one when this viewer has none.
  Future<RustFeedSession> open(
    FfiFeedSpec spec,
    NostrPublicKeyHex? viewer,
  ) async {
    final nativeSession = await _port.captureSession(viewer);
    final generation = await _adopt(nativeSession);
    final key = RustFeedSpecKey.fromSpec(spec);
    final opening =
        _live.remove(key) ?? _opening(spec, key, nativeSession, generation);
    _live[key] = opening;
    // Awaited before anything else runs: an open left unheard between
    // two microtasks reports its failure as an unhandled error.
    final session = await opening;
    _ensureCurrent(generation);
    await _evictOverflow();
    _ensureCurrent(generation);
    return session;
  }

  /// Drops one feed after a pull it could not answer: its watcher may
  /// be gone and its page died mid-plan, so the next pull starts fresh.
  Future<void> retire(RustFeedSession session) async {
    final live = await _resolved(_live[session.specKey]);
    if (!identical(live, session)) return;
    _live.remove(session.specKey);
    _pinCounts.remove(session);
    await _closed(session);
  }

  /// A passive watcher pins its session so unrelated pulls cannot evict and
  /// silently end the active query.
  void pin(RustFeedSession session) {
    _pinCounts.update(session, (count) => count + 1, ifAbsent: () => 1);
  }

  /// Releases one watcher pin and reports whether it was the last one.
  Future<bool> unpin(RustFeedSession session) async {
    final count = _pinCounts[session];
    if (count == null) return false;
    if (count > 1) {
      _pinCounts[session] = count - 1;
      return false;
    }
    _pinCounts.remove(session);
    await _evictOverflow();
    return true;
  }

  /// Retires the feeds whose snapshot stream ended — Rust closed them,
  /// or their watcher failed — so the next pull opens a fresh one.
  Future<void> retireDead() async {
    for (final opening in _live.values.toList()) {
      final session = await _resolved(opening);
      if (session != null && !session.isLive) await retire(session);
    }
  }

  /// Drops every open feed and the viewer-scoped rows behind them.
  Future<void> closeAll() async {
    final live = _live.values.toList();
    _live.clear();
    _pinCounts.clear();
    for (final opening in live) {
      await _closed(await _resolved(opening));
    }
  }

  /// Follows, mutes and outbox routing belong to the signed-in account
  /// and colour every feed the engine assembles, not only the main
  /// one, so a changed identity drops all of them.
  Future<int> _adopt(RustFeedAccountSession nativeSession) async {
    final currentSession = _nativeSession;
    if (currentSession != null &&
        nativeSession.generation.isBefore(currentSession.generation)) {
      throw StateError('The Rust feed account session is stale.');
    }
    if (currentSession == null || !nativeSession.hasSameOwner(currentSession)) {
      _nativeSession = nativeSession;
      _generation++;
      _adoption = closeAll();
    }
    final generation = _generation;
    await _adoption;
    _ensureCurrent(generation);
    return generation;
  }

  /// One open per spec even when two pulls race: both await the same
  /// handle. A failed open leaves nothing behind to serve later pulls.
  Future<RustFeedSession> _opening(
    FfiFeedSpec spec,
    RustFeedSpecKey key,
    RustFeedAccountSession nativeSession,
    int generation,
  ) async {
    try {
      return RustFeedSession(
        port: _port,
        feedId: await _port.openFeed(spec, nativeSession),
        specKey: key,
        deadline: _deadline,
      );
    } on Object {
      if (_generation == generation) _live.remove(key);
      rethrow;
    }
  }

  void _ensureCurrent(int generation) {
    if (_generation != generation) {
      throw StateError('The Rust feed account changed.');
    }
  }

  Future<void> _evictOverflow() async {
    while (_live.length > rustFeedLiveLimit) {
      if (!await _evictOldestUnpinned()) return;
    }
  }

  Future<bool> _evictOldestUnpinned() async {
    for (final entry in _live.entries.toList()) {
      final session = await _resolved(entry.value);
      if (session != null && _pinCounts.containsKey(session)) continue;
      if (!identical(_live[entry.key], entry.value)) continue;
      _live.remove(entry.key);
      await _closed(session);
      return true;
    }
    return false;
  }

  /// Awaits an open that may still be in flight; a failed one has
  /// nothing to close and has already dropped itself.
  Future<RustFeedSession?> _resolved(Future<RustFeedSession>? opening) async {
    try {
      return await opening;
    } on Object {
      return null;
    }
  }

  Future<void> _closed(RustFeedSession? session) async {
    try {
      await session?.close();
    } on Object catch (error, stackTrace) {
      log(
        'A Rust feed could not be closed.',
        name: 'ghostr.video.rustfeed',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
