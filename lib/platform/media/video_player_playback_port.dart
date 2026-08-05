import 'dart:async';
import 'dart:developer';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_surface_view.dart';
import 'package:ghostr/platform/media/video_player_value_listener.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';
import 'package:video_player/video_player.dart';

part 'video_player_controller_lifecycle.dart';
part 'video_player_buffering_overlay.dart';
part 'video_player_media_controller.dart';
part 'video_player_surface.dart';

class VideoPlayerPlaybackPort implements VideoPlaybackPort {
  const VideoPlayerPlaybackPort({
    VideoPlayerControllerDisposer controllerDisposer =
        disposeVideoPlayerController,
  }) : _controllerDisposer = controllerDisposer;

  final VideoPlayerControllerDisposer _controllerDisposer;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _VideoPlayerSurface(
      key: ValueKey(media.inventoryPlaybackIdentity),
      media: media,
      isActive: isActive,
      onPlaybackMediaReleased: onPlaybackMediaReleased,
      controllerDisposer: _controllerDisposer,
    );
  }
}
