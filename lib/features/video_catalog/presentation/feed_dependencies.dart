import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

/// Everything a feed needs from the outside world. The optional ports stay
/// optional: a feed without a social graph cannot block, and a feed without
/// a delivery engine or watch history simply stops reporting.
class FeedDependencies {
  const FeedDependencies({
    required this.feed,
    required this.engagement,
    this.optional = const FeedOptionalDependencies(),
  });

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final FeedOptionalDependencies optional;

  SocialGraphRepository? get social => optional.social;
  FeedFocusPort? get focus => optional.focus;
  WatchHistoryTracker? get watchTracker => optional.watchTracker;
}

/// Capabilities a feed can omit without changing its retrieval contract.
final class FeedOptionalDependencies {
  const FeedOptionalDependencies({
    this.social,
    this.focus,
    this.watchTracker,
  });

  final SocialGraphRepository? social;
  final FeedFocusPort? focus;
  final WatchHistoryTracker? watchTracker;
}
