import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

part 'gateway_video_playback_surface.dart';

/// Routes remote progressive media through the embedded loopback
/// gateway so playback starts on partial bytes; local files and
/// already-proxied streams stay on the delegate chain.
final class GatewayVideoPlaybackPort implements VideoPlaybackPort {
  const GatewayVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required ProgressivePlaybackGatewayPort gateway,
  })  : _delegate = delegate,
        _gateway = gateway;

  final VideoPlaybackPort _delegate;
  final ProgressivePlaybackGatewayPort _gateway;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    if (_isUnsupportedStream(media)) return const _UnsupportedStreamPanel();
    if (!_requiresProgressiveGateway(media)) {
      return _delegate.buildSurface(
        media: media,
        isActive: isActive,
        onPlaybackMediaReleased: onPlaybackMediaReleased,
      );
    }
    return _GatewayVideoPlaybackSurface(
      port: this,
      media: media,
      isActive: isActive,
      onPlaybackMediaReleased: onPlaybackMediaReleased,
    );
  }
}

bool _requiresProgressiveGateway(VideoMediaSource media) {
  return !media.isLocal &&
      media is! ProxiedProgressiveVideoMediaSource &&
      media.remoteDelivery == VideoMediaDelivery.progressive;
}

bool _isUnsupportedStream(VideoMediaSource media) {
  return !media.isLocal &&
      media is! ProxiedHlsVideoMediaSource &&
      media.remoteDelivery == VideoMediaDelivery.hls;
}
