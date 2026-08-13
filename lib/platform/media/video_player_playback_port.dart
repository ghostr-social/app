import 'dart:async';
import 'dart:developer';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/progressive_playback_refresh_port.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';
import 'package:ghostr/platform/media/video_player_playback_observer.dart';
import 'package:ghostr/platform/media/video_player_surface_view.dart';
import 'package:ghostr/platform/media/video_player_value_listener.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';
import 'package:video_player/video_player.dart';

part 'video_player_controller_lifecycle.dart';
part 'video_player_buffering_overlay.dart';
part 'video_player_media_controller.dart';
part 'video_player_playback_handoff.dart';
part 'video_player_surface.dart';
part 'video_player_surface_commands.dart';
part 'video_player_surface_loading.dart';
part 'video_player_surface_recovery.dart';
part 'video_player_surface_telemetry.dart';

class VideoPlayerPlaybackPort implements VideoPlaybackPort {
  VideoPlayerPlaybackPort({
    VideoPlayerControllerDisposer controllerDisposer =
        disposeVideoPlayerController,
    PlaybackTelemetryPort telemetry = const NoopPlaybackTelemetryPort(),
    PlaybackRecoveryPolicy recoveryPolicy =
        const PlaybackRecoveryPolicy.standard(),
    PlaybackScreenAwakePort screenAwake = const NoopPlaybackScreenAwakePort(),
  }) : _controllerDisposer = controllerDisposer,
       _telemetry = telemetry,
       _recoveryPolicy = recoveryPolicy,
       _screenAwake = screenAwake,
       _handoff = _VideoPlayerPlaybackHandoff();

  final VideoPlayerControllerDisposer _controllerDisposer;
  final PlaybackTelemetryPort _telemetry;
  final PlaybackRecoveryPolicy _recoveryPolicy;
  final PlaybackScreenAwakePort _screenAwake;
  final _VideoPlayerPlaybackHandoff _handoff;

  @override
  Widget buildSurface(VideoPlaybackSurfaceRequest request) {
    return _VideoPlayerSurface(
      key: ValueKey((request.media.inventoryPlaybackIdentity, request.videoId)),
      request: request,
      dependencies: _VideoPlayerSurfaceDependencies(
        controllerDisposer: _controllerDisposer,
        telemetry: _telemetry,
        recoveryPolicy: _recoveryPolicy,
        screenAwake: _screenAwake,
        handoff: _handoff,
      ),
    );
  }
}

final class _VideoPlayerSurfaceDependencies {
  const _VideoPlayerSurfaceDependencies({
    required this.controllerDisposer,
    required this.telemetry,
    required this.recoveryPolicy,
    required this.screenAwake,
    required this.handoff,
  });

  final VideoPlayerControllerDisposer controllerDisposer;
  final PlaybackTelemetryPort telemetry;
  final PlaybackRecoveryPolicy recoveryPolicy;
  final PlaybackScreenAwakePort screenAwake;
  final _VideoPlayerPlaybackHandoff handoff;
}
