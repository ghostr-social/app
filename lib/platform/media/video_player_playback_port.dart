import 'dart:async';
import 'dart:developer';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/progressive_playback_refresh_port.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_observer.dart';
import 'package:ghostr/platform/media/video_player_surface_view.dart';
import 'package:ghostr/platform/media/video_player_value_listener.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';
import 'package:video_player/video_player.dart';

part 'video_player_controller_lifecycle.dart';
part 'video_player_controller_budget.dart';
part 'video_player_controller_budget_pressure.dart';
part 'video_player_controller_settlement.dart';
part 'video_player_buffering_overlay.dart';
part 'video_player_initialization_settlement.dart';
part 'video_player_media_controller.dart';
part 'video_player_playback_handoff.dart';
part 'video_player_playback_handoff_command.dart';
part 'video_player_playback_handoff_reconciliation.dart';
part 'video_player_playback_handoff_state.dart';
part 'video_player_playback_handoff_teardown.dart';
part 'video_player_surface.dart';
part 'video_player_surface_capability_recovery.dart';
part 'video_player_surface_controller_acceptance.dart';
part 'video_player_surface_commands.dart';
part 'video_player_surface_failure.dart';
part 'video_player_surface_frame_correlation.dart';
part 'video_player_surface_loading.dart';
part 'video_player_surface_preparation_feedback.dart';
part 'video_player_surface_recovery.dart';
part 'video_player_surface_keys.dart';
part 'video_player_surface_telemetry.dart';
part 'video_player_teardown_gate.dart';

class VideoPlayerPlaybackPort
    implements
        VideoPlaybackPort,
        VideoPlaybackMemoryPressurePort,
        VideoPlaybackCapacityPort {
  VideoPlayerPlaybackPort({
    VideoPlayerControllerDisposer controllerDisposer =
        disposeVideoPlayerController,
    PlaybackTelemetryPort telemetry = const NoopPlaybackTelemetryPort(),
    PlaybackRecoveryPolicy recoveryPolicy =
        const PlaybackRecoveryPolicy.standard(),
    PlaybackScreenAwakePort screenAwake = const NoopPlaybackScreenAwakePort(),
    PlayerPreparationFeedbackPort preparationFeedback =
        const NoopPlayerPreparationFeedbackPort(),
    RenderedFirstFramePort renderedFirstFrames =
        const NoopRenderedFirstFramePort(),
  }) : _dependencies = _VideoPlayerSurfaceDependencies(
         controllerDisposer: controllerDisposer,
         telemetry: telemetry,
         recoveryPolicy: recoveryPolicy,
         screenAwake: screenAwake,
         preparationFeedback: preparationFeedback,
         renderedFirstFrames: renderedFirstFrames,
       );

  final _VideoPlayerSurfaceDependencies _dependencies;

  @override
  Widget buildSurface(VideoPlaybackSurfaceRequest request) {
    if (request.reservesPreparedDecoder || request.keepWarmWhenInactive) {
      _dependencies.controllerBudget.enableExtendedCapacity();
    }
    return _VideoPlayerSurface(
      key: _dependencies.surfaceKey(request),
      request: request,
      dependencies: _dependencies,
    );
  }

  @override
  void reportMemoryPressure() => _dependencies.controllerBudget.constrainTo(2);

  @override
  VideoPlaybackCapacitySnapshot get capacitySnapshot =>
      _dependencies.controllerBudget.snapshot;
}

final class _VideoPlayerSurfaceDependencies {
  _VideoPlayerSurfaceDependencies({
    required this.controllerDisposer,
    required this.telemetry,
    required this.recoveryPolicy,
    required this.screenAwake,
    required this.preparationFeedback,
    required this.renderedFirstFrames,
  }) : controllerBudget = _VideoPlayerControllerBudget(
         warpMaximumConcurrentPlaybackControllers,
         initialLimit: 2,
       ),
       handoff = _VideoPlayerPlaybackHandoff();

  final VideoPlayerControllerDisposer controllerDisposer;
  final PlaybackTelemetryPort telemetry;
  final PlaybackRecoveryPolicy recoveryPolicy;
  final PlaybackScreenAwakePort screenAwake;
  final PlayerPreparationFeedbackPort preparationFeedback;
  final RenderedFirstFramePort renderedFirstFrames;
  final _VideoPlayerControllerBudget controllerBudget;
  final _VideoPlayerPlaybackHandoff handoff;
  final Map<_ExactProgressiveSurfaceSlot, GlobalKey<_VideoPlayerSurfaceState>>
  _exactSurfaceKeys = {};
}
