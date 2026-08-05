import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';

enum VideoSearchPhase { loading, settled, failed }

/// One authoritative search result set adapted from a Rust feed revision.
final class VideoSearchSnapshot {
  const VideoSearchSnapshot({
    required this.revision,
    required this.phase,
    required this.page,
  });

  final BigInt revision;
  final VideoSearchPhase phase;
  final VideoFeedPage page;
}

/// Passive updates for one active search query.
abstract interface class VideoSearchUpdates {
  Stream<VideoSearchSnapshot> watchVideos(String query);
}
