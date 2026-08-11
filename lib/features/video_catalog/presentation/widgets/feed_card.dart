import 'dart:async';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_menu.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_metadata.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

export 'feed_card_action_rail.dart' show FeedCardActions, FeedCardShareStatus;

final class FeedCardPlayback {
  const FeedCardPlayback({required this.port, required this.isActive});

  final VideoPlaybackPort port;
  final bool isActive;
}

class FeedCard extends StatelessWidget {
  const FeedCard({
    required this.post,
    required this.playback,
    required this.actions,
    super.key,
  });

  final VideoPost post;
  final FeedCardPlayback playback;
  final FeedCardActions actions;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onLongPress: () => _openMenu(context),
      child: Stack(
        fit: StackFit.expand,
        children: [
          playback.port.buildSurface(
            media: post.media,
            videoId: PlaybackVideoId.parse(post.id),
            isActive: playback.isActive,
          ),
          const _FeedScrim(),
          _content(),
        ],
      ),
    );
  }

  void _openMenu(BuildContext context) {
    unawaited(
      showFeedCardMenu(
        context,
        post: post,
        onBlockCreator: actions.onBlockCreator,
      ),
    );
  }

  Widget _content() {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.md),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: FeedCardMetadata(
                post: post,
                onOpenHashtag: actions.onOpenHashtag,
              ),
            ),
            const SizedBox(width: AppSpacing.md),
            FeedCardActionRail(post: post, actions: actions),
          ],
        ),
      ),
    );
  }
}

class _FeedScrim extends StatelessWidget {
  const _FeedScrim();

  @override
  Widget build(BuildContext context) {
    return const DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [AppPalette.videoScrimTop, AppPalette.videoScrimBottom],
        ),
      ),
    );
  }
}
