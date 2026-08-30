import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/video_loading_surface.dart';

part 'hls_video_playback_surface.dart';
part 'hls_video_playback_lease_surface.dart';

final class HlsVideoPlaybackPort
    implements VideoPlaybackPort, VideoPlaybackMemoryPressurePort {
  const HlsVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required HlsPlaybackGatewayPort gateway,
  }) : _delegate = delegate,
       _gateway = gateway;

  final VideoPlaybackPort _delegate;
  final HlsPlaybackGatewayPort _gateway;

  @override
  Widget buildSurface(VideoPlaybackSurfaceRequest request) {
    final media = request.media;
    if (!_requiresGateway(media)) {
      return _delegate.buildSurface(request);
    }
    return _HlsVideoPlaybackSurface(
      port: this,
      request: VideoPlaybackSurfaceRequest(
        media: media,
        videoId: request.videoId,
        isActive: request.isActive,
        mode: request.mode,
        surfaceScope: request.surfaceScope,
        reservesPreparedDecoder: request.reservesPreparedDecoder,
        keepWarmWhenInactive: request.keepWarmWhenInactive,
        hlsAuthority: request.hlsAuthority,
        onHlsFirstFrameRendered: request.onHlsFirstFrameRendered,
        onPlaybackMediaReleased: request.onPlaybackMediaReleased,
        preview: request.preview,
      ),
    );
  }

  @override
  void reportMemoryPressure() {
    final delegate = _delegate;
    if (delegate is VideoPlaybackMemoryPressurePort) {
      (delegate as VideoPlaybackMemoryPressurePort).reportMemoryPressure();
    }
  }
}

bool _requiresGateway(VideoMediaSource media) {
  return !media.isLocal &&
      media is! ProxiedHlsVideoMediaSource &&
      media.remoteDelivery == VideoMediaDelivery.hls;
}
