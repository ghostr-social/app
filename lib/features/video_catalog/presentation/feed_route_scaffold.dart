import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

/// What a feed pushed as its own route needs from its host: the playback
/// and sharing ports, and where its profile and hashtag links lead.
abstract base class RoutedFeedRequest {
  const RoutedFeedRequest({
    required this.playbackPort,
    required this.createComments,
    required this.shareWorkflow,
    required this.onOpenProfile,
    required this.onOpenHashtag,
  });

  /// Names what the feed is showing in the app bar.
  String get title;

  final VideoPlaybackPort playbackPort;
  final CommentsCubit Function(VideoPost post) createComments;
  final VideoShareWorkflow shareWorkflow;
  final Future<void> Function(ProfileId) onOpenProfile;
  final Future<void> Function(String) onOpenHashtag;
}

/// Scaffolds a routed feed and pauses it only behind opaque destinations.
class FeedRouteScaffold extends StatelessWidget {
  const FeedRouteScaffold({required this.request, super.key});

  final RoutedFeedRequest request;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(request.title)),
      body: FeedScreen(
        bindings: FeedScreenBindings(
          onOpenProfile: (profileId) => _openProfile(context, profileId),
          onOpenHashtag: (hashtag) => _openHashtag(context, hashtag),
          playbackPort: request.playbackPort,
          shareWorkflow: request.shareWorkflow,
          createComments: request.createComments,
          isActive: TickerMode.valuesOf(context).enabled,
        ),
      ),
    );
  }

  void _openProfile(BuildContext context, ProfileId profileId) {
    unawaited(_openAndRefresh(context, () => request.onOpenProfile(profileId)));
  }

  void _openHashtag(BuildContext context, String hashtag) {
    unawaited(_openAndRefresh(context, () => request.onOpenHashtag(hashtag)));
  }

  Future<void> _openAndRefresh(
    BuildContext context,
    Future<void> Function() open,
  ) async {
    await open();
    if (context.mounted) await context.read<FeedCubit>().refresh();
  }
}
