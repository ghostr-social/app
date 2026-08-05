import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

enum VideoFeedUpdatePhase { loading, settled, failed }

/// A revision signal from the Rust-owned feed. Posts stay behind the outer
/// feed repository so watch history, blocking, local merge, and hydration
/// remain authoritative.
final class VideoFeedUpdate {
  const VideoFeedUpdate({
    required this.revision,
    required this.phase,
    required this.hasPosts,
  });

  final BigInt revision;
  final VideoFeedUpdatePhase phase;
  final bool hasPosts;
}

abstract interface class VideoFeedUpdates {
  Stream<VideoFeedUpdate> watchFeed(FeedKind kind);
}

/// Optional scope check for update sources whose native feed is expensive
/// to replace.
abstract interface class VideoFeedUpdateRefreshPolicy {
  Future<bool> shouldRebind(FeedKind kind);
}
