import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

/// Everything a feed needs from the outside world. The optional ports stay
/// optional: a feed without a social graph cannot block, and a feed without
/// a delivery engine or watch history simply stops reporting.
class FeedDependencies {
  const FeedDependencies({
    required this.feed,
    required this.engagement,
    this.viewerId,
    this.followProfile,
    this.optional = const FeedOptionalDependencies(),
  });

  final ProfileId? viewerId;
  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final FollowProfileWorkflow? followProfile;
  final FeedOptionalDependencies optional;

  SocialGraphRepository? get social => optional.social;
  FeedFocusPort? get focus => optional.focus;
  WatchHistoryTracker? get watchTracker => optional.watch.tracker;
  VideoFeedUpdates? get updates => optional.updates;
  VideoDeliveryUpdates? get deliveryUpdates => optional.deliveryUpdates;
  PlaybackPreparationUpdates? get preparationUpdates =>
      optional.delivery.preparationUpdates;
}

/// Capabilities a feed can omit without changing its retrieval contract.
final class FeedOptionalDependencies {
  const FeedOptionalDependencies({
    this.social,
    this.focus,
    this.watch = const FeedWatchDependencies(),
    this.delivery = const FeedDeliveryDependencies(),
  });

  final SocialGraphRepository? social;
  final FeedFocusPort? focus;
  final FeedWatchDependencies watch;
  final FeedDeliveryDependencies delivery;

  VideoFeedUpdates? get updates => delivery.updates;
  VideoRepostRepository? get reposts => delivery.reposts;
  VideoDeliveryUpdates? get deliveryUpdates => delivery.deliveryUpdates;
}

final class FeedWatchDependencies {
  const FeedWatchDependencies({this.tracker});

  final WatchHistoryTracker? tracker;
}

final class FeedDeliveryDependencies {
  const FeedDeliveryDependencies({
    this.updates,
    this.reposts,
    this.deliveryUpdates,
    this.preparationUpdates,
  });

  final VideoFeedUpdates? updates;
  final VideoRepostRepository? reposts;
  final VideoDeliveryUpdates? deliveryUpdates;
  final PlaybackPreparationUpdates? preparationUpdates;
}
