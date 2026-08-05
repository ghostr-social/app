import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

part 'gateway_video_playback_surface.dart';

/// Routes remote progressive media through the embedded loopback
/// gateway so playback starts on partial bytes; local files and
/// already-proxied streams stay on the delegate chain.
final class GatewayVideoPlaybackPort implements VideoPlaybackPort {
  GatewayVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required ProgressivePlaybackGatewayPort gateway,
  })  : _delegate = delegate,
        _createCubit = ((media) => GatewayPlaybackCubit(gateway, media));

  final VideoPlaybackPort _delegate;
  final GatewayPlaybackCubit Function(VideoMediaSource) _createCubit;

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
      delegate: _delegate,
      createCubit: _createCubit,
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
