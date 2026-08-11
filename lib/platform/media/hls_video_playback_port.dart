import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

part 'hls_video_playback_surface.dart';
part 'hls_video_playback_lease_surface.dart';

final class HlsVideoPlaybackPort implements VideoPlaybackPort {
  const HlsVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required HlsPlaybackGatewayPort gateway,
  }) : _delegate = delegate,
       _gateway = gateway;

  final VideoPlaybackPort _delegate;
  final HlsPlaybackGatewayPort _gateway;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    PlaybackVideoId? videoId,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    if (!_requiresGateway(media)) {
      return _delegate.buildSurface(
        media: media,
        videoId: videoId,
        isActive: isActive,
        onPlaybackMediaReleased: onPlaybackMediaReleased,
      );
    }
    return _HlsVideoPlaybackSurface(
      port: this,
      request: VideoPlaybackSurfaceRequest(
        media: media,
        videoId: videoId,
        isActive: isActive,
        onPlaybackMediaReleased: onPlaybackMediaReleased,
      ),
    );
  }
}

bool _requiresGateway(VideoMediaSource media) {
  return !media.isLocal &&
      media is! ProxiedHlsVideoMediaSource &&
      media.remoteDelivery == VideoMediaDelivery.hls;
}
