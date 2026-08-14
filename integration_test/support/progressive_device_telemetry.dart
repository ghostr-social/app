import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';

import 'device_playback_probe.dart';

final class ProgressiveDeviceTelemetry implements PlaybackTelemetryPort {
  final rust = FfiPlaybackTelemetryPort();
  final probe = DevicePlaybackProbe();
  final observedDeliveryIds = <String>{};

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) {
    return rust.openSession(videoId, deliveryId);
  }

  @override
  void activate(PlaybackSession session) {
    observedDeliveryIds.add(session.deliveryId.value);
    rust.activate(session);
    probe.activate(session);
  }

  @override
  void report(PlaybackObservation observation) {
    rust.report(observation);
    probe.report(observation);
  }

  @override
  void deactivate(PlaybackSession session) {
    rust.deactivate(session);
    probe.deactivate(session);
  }
}
