import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';

void main() {
  test('separate ports share a process-monotonic session generation', () {
    final first = FfiPlaybackTelemetryPort().openSession(
      PlaybackVideoId.parse('first'),
      PlaybackDeliveryId.parse('delivery-first'),
    );
    final second = FfiPlaybackTelemetryPort().openSession(
      PlaybackVideoId.parse('second'),
      PlaybackDeliveryId.parse('delivery-second'),
    );

    expect(second.generation, greaterThan(first.generation));
  });
}
