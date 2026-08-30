import 'dart:async';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_menu.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_metadata.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_video_interaction.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

export 'feed_card_actions.dart';

final class FeedCardPlayback {
  const FeedCardPlayback({
    required this.port,
    required this.source,
    required this.isActive,
    this.surfaceScope,
    this.preparedOnly = false,
    this.keepWarmWhenInactive = false,
    this.hlsAuthority,
    this.onHlsFirstFrameRendered,
    this.onPlaybackMediaReleased,
  });

  final VideoPlaybackPort port;
  final FeedCardPlaybackSource source;
  final bool isActive;
  final VideoPlaybackSurfaceScope? surfaceScope;
  final bool preparedOnly;
  final bool keepWarmWhenInactive;
  final HlsPlaybackAuthority? hlsAuthority;
  final ValueChanged<HlsPlaybackAuthority>? onHlsFirstFrameRendered;
  final VoidCallback? onPlaybackMediaReleased;
}

final class FeedCardPlaybackSource {
  const FeedCardPlaybackSource.direct(this.media) : _prepared = null;

  FeedCardPlaybackSource.prepared(PreparedProgressivePlayback prepared)
    : media = prepared.origin,
      _prepared = prepared;

  final VideoMediaSource media;
  final PreparedProgressivePlayback? _prepared;

  VideoPlaybackSurfaceRequest decorate(VideoPlaybackSurfaceRequest request) {
    final prepared = _prepared;
    return prepared == null
        ? request
        : PreparedProgressiveVideoPlaybackRequest(
            request: request,
            prepared: prepared,
          );
  }
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
    return ExcludeSemantics(
      excluding: playback.preparedOnly,
      child: IgnorePointer(
        ignoring: playback.preparedOnly,
        child: FeedVideoInteraction(
          key: ValueKey(post.id.value),
          isActive: playback.isActive,
          onOpenMenu: () => _openMenu(context),
          surfaceBuilder: (mode) => playback.port.buildSurface(
            playback.source.decorate(
              VideoPlaybackSurfaceRequest(
                media: playback.source.media,
                videoId: PlaybackVideoId.parse(post.id),
                isActive: playback.isActive,
                mode: mode,
                surfaceScope: playback.surfaceScope,
                keepWarmWhenInactive: playback.keepWarmWhenInactive,
                hlsAuthority: playback.hlsAuthority,
                onHlsFirstFrameRendered: playback.onHlsFirstFrameRendered,
                onPlaybackMediaReleased: playback.onPlaybackMediaReleased,
              ),
            ),
          ),
          overlay: playback.preparedOnly
              ? const SizedBox.shrink()
              : Stack(
                  fit: StackFit.expand,
                  children: [const _FeedScrim(), _content()],
                ),
        ),
      ),
    );
  }

  void _openMenu(BuildContext context) {
    unawaited(
      showFeedCardMenu(
        context,
        post: post,
        onBlockCreator: actions.moderation.onBlockCreator,
      ),
    );
  }

  Widget _content() {
    return SafeArea(
      child: LayoutBuilder(
        builder: (_, constraints) => _overlay(constraints.maxHeight),
      ),
    );
  }

  Widget _overlay(double height) {
    return Padding(
      padding: const EdgeInsets.all(AppSpacing.md),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Expanded(
            child: FeedCardMetadata(
              post: post,
              onOpenHashtag: actions.navigation.onOpenHashtag,
            ),
          ),
          const SizedBox(width: AppSpacing.md),
          _rail(height - AppSpacing.md * 2),
        ],
      ),
    );
  }

  Widget _rail(double height) {
    final rail = FeedCardActionRail(post: post, actions: actions);
    if (height >= AppSize.feedRailMinHeight) return rail;
    return SizedBox(
      height: height,
      child: SingleChildScrollView(reverse: true, child: rail),
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
