import 'dart:async';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

typedef LiveRustFeedSessionCapture = Future<RustFeedAccountSession> Function(
  NostrPublicKeyHex? account,
);

/// A Rust feed port whose feeds stay open like the engine's do: every
/// open gets its own handle and its own snapshot stream, and a test can
/// publish later revisions on it — the background pages Rust files into
/// an open feed while nobody is pulling.
class LiveRustFeedPort implements RustFeedPort {
  LiveRustFeedPort({
    this.firstPage = const <FfiFeedUpdate>[],
    this.sessionCapture,
  });

  /// The snapshots every newly opened feed publishes straight away.
  List<FfiFeedUpdate> firstPage;
  final LiveRustFeedSessionCapture? sessionCapture;
  bool moreAvailable = true;
  BigInt sessionGeneration = BigInt.zero;

  final List<FfiFeedSpec> openedSpecs = <FfiFeedSpec>[];
  final List<RustFeedId> closedFeedIds = <RustFeedId>[];
  final List<BigInt?> loadMoreCursors = <BigInt?>[];
  final Map<RustFeedId, StreamController<FfiFeedUpdate>> _feeds =
      <RustFeedId, StreamController<FfiFeedUpdate>>{};

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) async {
    final capture = sessionCapture;
    if (capture != null) return capture(expectedAccount);
    return RustFeedAccountSession(
      account: expectedAccount,
      generation: RustNostrSessionGeneration.fromBridge(sessionGeneration),
    );
  }

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    openedSpecs.add(spec);
    final feedId = RustFeedId.parse('${openedSpecs.length}');
    _feeds[feedId] = StreamController<FfiFeedUpdate>();
    for (final update in firstPage) {
      publish(feedId, update);
    }
    return feedId;
  }

  /// One snapshot on an open feed, as the engine's watcher publishes
  /// them (rust/src/api/feed_updates_stream.rs).
  void publish(RustFeedId feedId, FfiFeedUpdate update) {
    _feeds[feedId]?.add(update);
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) {
    return _feeds[feedId]!.stream;
  }

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) async {
    loadMoreCursors.add(olderThanSecs);
    return moreAvailable;
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) async {
    closedFeedIds.add(feedId);
    await _feeds.remove(feedId)?.close();
  }
}
