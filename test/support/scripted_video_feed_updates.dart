import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

typedef VideoFeedStreamFactory = Stream<VideoFeedUpdate> Function();

final class ScriptedVideoFeedUpdates implements VideoFeedUpdates {
  ScriptedVideoFeedUpdates(List<VideoFeedStreamFactory> attempts)
    : _attempts = List<VideoFeedStreamFactory>.of(attempts);

  final List<VideoFeedStreamFactory> _attempts;
  int watchCalls = 0;

  @override
  Stream<VideoFeedUpdate> watchFeed(FeedKind kind) {
    watchCalls += 1;
    return _attempts.removeAt(0)();
  }
}
