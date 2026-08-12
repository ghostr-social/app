import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_coordinator.dart';

import '../support/recording_screen_awake_port.dart';

void main() {
  test('screen stays awake until the last demanding surface stops', () {
    final screen = RecordingScreenAwakePort();
    final coordinator = PlaybackScreenAwakeCoordinator(screen);
    final outgoing = Object();
    final incoming = Object();

    coordinator.observePhase(outgoing, PlaybackPhase.playing);
    coordinator.observePhase(incoming, PlaybackPhase.starting);
    expect(screen.toggles, [true]);

    coordinator.release(outgoing);
    expect(screen.toggles, [true]);

    coordinator.observePhase(incoming, PlaybackPhase.ended);
    expect(screen.toggles, [true, false]);
  });
}
