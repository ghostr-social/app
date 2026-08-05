import 'dart:async';

import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

final class ControllableVideoFeedUpdates implements VideoFeedUpdates {
  ControllableVideoFeedUpdates() {
    _controller = StreamController<VideoFeedUpdate>.broadcast(
      onCancel: () => cancellations += 1,
    );
  }

  late final StreamController<VideoFeedUpdate> _controller;
  final watchedKinds = <FeedKind>[];
  int cancellations = 0;

  @override
  Stream<VideoFeedUpdate> watchFeed(FeedKind kind) {
    watchedKinds.add(kind);
    return _controller.stream;
  }

  void add(VideoFeedUpdate update) => _controller.add(update);

  Future<void> close() => _controller.close();
}
