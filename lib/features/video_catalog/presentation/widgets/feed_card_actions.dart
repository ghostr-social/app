import 'package:flutter/foundation.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

enum FeedCardShareStatus { available, unavailable, downloading, busy }

final class FeedCardNavigationActions {
  const FeedCardNavigationActions({
    required this.onOpenProfile,
    required this.onOpenComments,
    required this.onOpenHashtag,
    this.onFollowCreator,
  });

  final VoidCallback onOpenProfile;
  final VoidCallback onOpenComments;
  final ValueChanged<String> onOpenHashtag;
  final Future<void> Function(ProfileSummary creator)? onFollowCreator;
}

final class FeedCardEngagementActions {
  const FeedCardEngagementActions({
    required this.onToggleLike,
    this.onToggleRepost,
  });

  final Future<void> Function(VideoPost post) onToggleLike;
  final Future<void> Function(VideoPost post)? onToggleRepost;
}

final class FeedCardModerationActions {
  const FeedCardModerationActions({required this.onBlockCreator});

  final VoidCallback onBlockCreator;
}

final class FeedCardSharingActions {
  const FeedCardSharingActions({
    required this.onShare,
    this.status = FeedCardShareStatus.available,
  });

  final Future<void> Function(VideoPost post, VideoShareOrigin origin) onShare;
  final FeedCardShareStatus status;
}

final class FeedCardActions {
  const FeedCardActions({
    required this.navigation,
    required this.engagement,
    required this.moderation,
    required this.sharing,
  });

  final FeedCardNavigationActions navigation;
  final FeedCardEngagementActions engagement;
  final FeedCardModerationActions moderation;
  final FeedCardSharingActions sharing;
}
