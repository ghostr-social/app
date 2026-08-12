import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_coordinator.dart';

import '../support/recording_screen_awake_port.dart';

void main() {
  test('screen wakes while one surface demands it and sleeps after', () {
    final screen = RecordingScreenAwakePort();
    final coordinator = PlaybackScreenAwakeCoordinator(screen);
    final surface = Object();

    coordinator.observePhase(surface, PlaybackPhase.starting);
    coordinator.observePhase(surface, PlaybackPhase.playing);
    coordinator.observePhase(surface, PlaybackPhase.networkStalled);
    expect(screen.toggles, [true]);

    coordinator.observePhase(surface, PlaybackPhase.paused);
    expect(screen.toggles, [true, false]);

    coordinator.observePhase(surface, PlaybackPhase.playing);
    coordinator.release(surface);
    expect(screen.toggles, [true, false, true, false]);

    coordinator.release(surface);
    coordinator.observePhase(surface, PlaybackPhase.inactive);
    expect(screen.toggles, [true, false, true, false]);
  });
}
