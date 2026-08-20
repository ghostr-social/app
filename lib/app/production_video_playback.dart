import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_coordinator.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/unsupported_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/platform/media/wakelock_screen_awake.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

/// Progressive remote media streams from the embedded loopback gateway;
/// the inventory full-download gate no longer fronts playback.
VideoPlaybackPort buildProductionVideoPlayback(
  ProductionVideoDelivery delivery, {
  ProgressivePlaybackGatewayPort progressiveGateway =
      const FfiProgressivePlaybackGateway(),
  PlaybackTelemetryPort? playbackTelemetry,
  PlayerPreparationFeedbackPort? playerPreparationFeedback,
  RenderedFirstFramePort? renderedFirstFrames,
  PlaybackScreenAwakePort? screenAwake,
}) {
  if (!delivery.playbackCapabilities.supportsAny) {
    return const UnsupportedVideoPlaybackPort();
  }
  final progressivePlayback = GatewayVideoPlaybackPort(
    delegate: VideoPlayerPlaybackPort(
      telemetry: playbackTelemetry ?? FfiPlaybackTelemetryPort(),
      preparationFeedback:
          playerPreparationFeedback ?? FfiPlayerPreparationFeedbackPort(),
      renderedFirstFrames:
          renderedFirstFrames ?? NativeRenderedFirstFramePort.production(),
      screenAwake:
          screenAwake ??
          PlaybackScreenAwakeCoordinator(const WakelockScreenAwake()),
    ),
    gateway: progressiveGateway,
  );
  final gateway = delivery.hlsPlaybackGateway;
  if (gateway == null || !delivery.playbackCapabilities.supportsHls) {
    return progressivePlayback;
  }
  return HlsVideoPlaybackPort(delegate: progressivePlayback, gateway: gateway);
}
