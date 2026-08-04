import 'dart:async';

import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// A Rust feed port whose feeds stay open like the engine's do: every
/// open gets its own handle and its own snapshot stream, and a test can
/// publish later revisions on it — the background pages Rust files into
/// an open feed while nobody is pulling.
class LiveRustFeedPort implements RustFeedPort {
  LiveRustFeedPort({this.firstPage = const <FfiFeedUpdate>[]});

  /// The snapshots every newly opened feed publishes straight away.
  List<FfiFeedUpdate> firstPage;
  bool moreAvailable = true;

  final List<FfiFeedSpec> openedSpecs = <FfiFeedSpec>[];
  final List<String> closedFeedIds = <String>[];
  final List<BigInt?> loadMoreCursors = <BigInt?>[];
  final Map<String, StreamController<FfiFeedUpdate>> _feeds =
      <String, StreamController<FfiFeedUpdate>>{};

  @override
  Future<String> openFeed(FfiFeedSpec spec) async {
    openedSpecs.add(spec);
    final feedId = '${openedSpecs.length}';
    _feeds[feedId] = StreamController<FfiFeedUpdate>();
    for (final update in firstPage) {
      publish(feedId, update);
    }
    return feedId;
  }

  /// One snapshot on an open feed, as the engine's watcher publishes
  /// them (rust/src/api/feed_updates_stream.rs).
  void publish(String feedId, FfiFeedUpdate update) {
    _feeds[feedId]?.add(update);
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(String feedId) => _feeds[feedId]!.stream;

  @override
  Future<bool> loadMore(String feedId, {BigInt? olderThanSecs}) async {
    loadMoreCursors.add(olderThanSecs);
    return moreAvailable;
  }

  @override
  Future<void> closeFeed(String feedId) async {
    closedFeedIds.add(feedId);
    await _feeds.remove(feedId)?.close();
  }
}
