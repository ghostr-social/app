import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

/// The authoritative feed projection and its optional Rust revision signal.
final class VideoFeedBinding {
  const VideoFeedBinding({required this.repository, this.updates});

  final VideoFeedRepository repository;
  final VideoFeedUpdates? updates;
}
