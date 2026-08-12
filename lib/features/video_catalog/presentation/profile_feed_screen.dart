import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_route_scaffold.dart';

final class ProfileFeedRequest extends RoutedFeedRequest {
  const ProfileFeedRequest({
    required this.creator,
    required super.playbackPort,
    required super.createComments,
    required super.shareWorkflow,
    required super.onOpenProfile,
    required super.onOpenHashtag,
  });

  /// Whose shelf of published videos the feed plays.
  final ProfileSummary creator;

  @override
  String get title => creator.displayName;
}

/// One creator's published videos as a full swipeable feed — the same
/// player, likes, and comments as the home feed, opened on the video the
/// viewer tapped on the profile grid.
class ProfileFeedScreen extends StatelessWidget {
  const ProfileFeedScreen({required this.request, super.key});

  final ProfileFeedRequest request;

  @override
  Widget build(BuildContext context) => FeedRouteScaffold(request: request);
}
