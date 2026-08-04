import 'dart:developer';

import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_session.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// How many Rust feeds one source holds open at a time. Reopening a
/// feed costs a cold relay round trip, so feeds stay; but each one
/// keeps a bounded row window in the engine
/// (`FEED_POST_RETENTION`, rust/src/discovery/feed_store.rs) and one
/// snapshot stream, so a source keeps only what a viewer moves
/// between: the feed they are in, the one they came from, and the
/// search or profile they just opened.
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
  final Map<String, Future<RustFeedSession>> _live =
      <String, Future<RustFeedSession>>{};
  String? _viewer;
  bool _adopted = false;

  /// The live feed for [spec], opening one when this viewer has none.
  Future<RustFeedSession> open(FfiFeedSpec spec, String? viewer) async {
    await _adopt(viewer);
    final key = _keyOf(spec);
    final opening = _live.remove(key) ?? _opening(spec, key);
    _live[key] = opening;
    // Awaited before anything else runs: an open left unheard between
    // two microtasks reports its failure as an unhandled error.
    final session = await opening;
    await _evictOverflow();
    return session;
  }

  /// Drops one feed after a pull it could not answer: its watcher may
  /// be gone and its page died mid-plan, so the next pull starts fresh.
  Future<void> retire(RustFeedSession session) async {
    final live = await _resolved(_live[session.specKey]);
    if (!identical(live, session)) return;
    _live.remove(session.specKey);
    await _closed(session);
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
    for (final opening in live) {
      await _closed(await _resolved(opening));
    }
  }

  /// Follows, mutes and outbox routing belong to the signed-in account
  /// and colour every feed the engine assembles, not only the main
  /// one, so a changed identity drops all of them.
  Future<void> _adopt(String? viewer) async {
    if (_adopted && _viewer == viewer) return;
    _adopted = true;
    _viewer = viewer;
    await closeAll();
  }

  /// One open per spec even when two pulls race: both await the same
  /// handle. A failed open leaves nothing behind to serve later pulls.
  Future<RustFeedSession> _opening(FfiFeedSpec spec, String key) async {
    try {
      return RustFeedSession(
        port: _port,
        feedId: await _port.openFeed(spec),
        specKey: key,
        deadline: _deadline,
      );
    } on Object {
      _live.remove(key);
      rethrow;
    }
  }

  Future<void> _evictOverflow() async {
    while (_live.length > rustFeedLiveLimit) {
      final oldest = _live.remove(_live.keys.first);
      await _closed(await _resolved(oldest));
    }
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

  /// One key per feed the engine can serve. `FfiFeedSpec` equality is
  /// identity-based on its creator list, so the parts are joined
  /// instead — on NUL, which no relay query or hex key carries.
  String _keyOf(FfiFeedSpec spec) {
    return <String>[
      spec.kind,
      spec.value ?? '',
      spec.viewerPubkey ?? '',
      ...spec.creators,
    ].join('\u0000');
  }
}
