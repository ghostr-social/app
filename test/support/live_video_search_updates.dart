import 'dart:async';

import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';

final class LiveVideoSearchUpdates implements VideoSearchUpdates {
  final Map<String, StreamController<VideoSearchSnapshot>> _queries = {};
  final Map<String, BigInt> _revisions = {};

  @override
  Stream<VideoSearchSnapshot> watchVideos(String query) {
    return _queries
        .putIfAbsent(query, StreamController<VideoSearchSnapshot>.broadcast)
        .stream;
  }

  void publish(
    String query,
    VideoFeedPage page, {
    VideoSearchPhase phase = VideoSearchPhase.settled,
  }) {
    final revision = (_revisions[query] ?? BigInt.zero) + BigInt.one;
    _revisions[query] = revision;
    _queries[query]?.add(VideoSearchSnapshot(
      revision: revision,
      phase: phase,
      page: page,
    ));
  }

  void fail(String query, Object error) {
    _queries[query]?.addError(error);
  }

  Future<void> close() async {
    for (final controller in _queries.values) {
      await controller.close();
    }
  }
}
