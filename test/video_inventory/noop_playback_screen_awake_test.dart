import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_port.dart';

void main() {
  test('default screen awake port ignores the playback lifecycle', () {
    // Deliberately non-const so the constructor line executes at runtime.
    final port = NoopPlaybackScreenAwakePort();
    final surface = Object();

    port.observePhase(surface, PlaybackPhase.playing);
    port.release(surface);

    expect(port, isA<PlaybackScreenAwakePort>());
  });
}
