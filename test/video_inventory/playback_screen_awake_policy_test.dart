import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';

void main() {
  test('only phases that advance media demand an awake screen', () {
    const demanding = [
      PlaybackPhase.starting,
      PlaybackPhase.playing,
      PlaybackPhase.networkStalled,
    ];
    const resting = [
      PlaybackPhase.paused,
      PlaybackPhase.ended,
      PlaybackPhase.failed,
      PlaybackPhase.inactive,
    ];

    expect(demanding.every((phase) => phase.keepsScreenAwake), isTrue);
    expect(resting.any((phase) => phase.keepsScreenAwake), isFalse);
    expect({...demanding, ...resting}, PlaybackPhase.values.toSet());
  });
}
