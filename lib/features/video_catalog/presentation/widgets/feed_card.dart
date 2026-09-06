import 'dart:async';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_actions.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_overlay.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_menu.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_video_interaction.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

export 'feed_card_actions.dart';
export 'feed_card_overlay.dart';

final class FeedCardPlayback {
  const FeedCardPlayback({
    required this.port,
    required this.source,
    required this.isActive,
    this.surfaceScope,
    this.preparedOnly = false,
    this.keepWarmWhenInactive = false,
    this.reservesPreparedDecoder = false,
    this.hlsAuthority,
    this.onHlsFirstFrameRendered,
    this.onHlsDecodedReadinessRevoked,
  });

  final VideoPlaybackPort port;
  final FeedCardPlaybackSource source;
  final bool isActive;
  final VideoPlaybackSurfaceScope? surfaceScope;
  final bool preparedOnly;
  final bool keepWarmWhenInactive;
  final bool reservesPreparedDecoder;
  final HlsPlaybackAuthority? hlsAuthority;
  final ValueChanged<HlsPlaybackAuthority>? onHlsFirstFrameRendered;
  final ValueChanged<HlsPlaybackAuthority>? onHlsDecodedReadinessRevoked;
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
    this.showOverlay = true,
    super.key,
  });

  final VideoPost post;
  final FeedCardPlayback playback;
  final FeedCardActions actions;
  final bool showOverlay;

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
                reservesPreparedDecoder: playback.reservesPreparedDecoder,
                hlsAuthority: playback.hlsAuthority,
                onHlsFirstFrameRendered: playback.onHlsFirstFrameRendered,
                onHlsDecodedReadinessRevoked:
                    playback.onHlsDecodedReadinessRevoked,
              ),
            ),
          ),
          overlay: playback.preparedOnly || !showOverlay
              ? const SizedBox.shrink()
              : FeedCardOverlay(post: post, actions: actions),
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
}
