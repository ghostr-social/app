import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_route_scaffold.dart';

final class DiscoveryFeedRequest extends RoutedFeedRequest {
  const DiscoveryFeedRequest({
    required this.query,
    required super.playbackPort,
    required super.createComments,
    required super.shareWorkflow,
    required super.onOpenProfile,
    required super.onOpenHashtag,
  });

  final String query;

  @override
  String get title => query;
}

/// A full swipeable video feed for one search query or `#hashtag` — the
/// same player, likes, comments, and endless paging as the home feed.
class DiscoveryFeedScreen extends StatelessWidget {
  const DiscoveryFeedScreen({required this.request, super.key});

  final DiscoveryFeedRequest request;

  @override
  Widget build(BuildContext context) => FeedRouteScaffold(request: request);
}
