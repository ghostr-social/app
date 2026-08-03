import 'package:flutter/material.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class DiscoveryFeedRequest {
  const DiscoveryFeedRequest({
    required this.query,
    required this.playbackPort,
    required this.createComments,
    required this.onOpenProfile,
    required this.onOpenHashtag,
  });

  final String query;
  final VideoPlaybackPort playbackPort;
  final CommentsCubit Function(VideoPost post) createComments;
  final ValueChanged<ProfileId> onOpenProfile;
  final ValueChanged<String> onOpenHashtag;
}

/// A full swipeable video feed for one search query or `#hashtag` — the
/// same player, likes, comments, and endless paging as the home feed.
class DiscoveryFeedScreen extends StatelessWidget {
  const DiscoveryFeedScreen({required this.request, super.key});

  final DiscoveryFeedRequest request;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(request.query)),
      body: FeedScreen(
        bindings: FeedScreenBindings(
          onOpenProfile: request.onOpenProfile,
          onOpenHashtag: request.onOpenHashtag,
          playbackPort: request.playbackPort,
          createComments: request.createComments,
          // Playback pauses whenever another route covers this feed.
          isActive: ModalRoute.of(context)?.isCurrent ?? true,
        ),
      ),
    );
  }
}
