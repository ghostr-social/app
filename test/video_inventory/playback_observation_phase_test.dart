import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

void main() {
  test('playback phases make only measured network stalls emergencies', () {
    final videoId = PlaybackVideoId.parse(' clip ');
    final deliveryId = PlaybackDeliveryId.parse('delivery');
    final session = PlaybackSession(videoId, deliveryId, 7);
    final observation = PlaybackObservation(
      session: session,
      phase: PlaybackPhase.networkStalled,
      metrics: PlaybackMetrics(
        position: const Duration(seconds: 2),
        bufferedExtent: const Duration(seconds: 5),
        playbackRate: 1,
      ),
    );

    expect(videoId.value, 'clip');
    expect(observation.videoId, videoId);
    expect(observation.bufferAhead, const Duration(seconds: 3));
    expect(observation.phase.isNetworkStall, isTrue);
    expect(
      [
        PlaybackPhase.starting,
        PlaybackPhase.playing,
        PlaybackPhase.paused,
        PlaybackPhase.ended,
        PlaybackPhase.inactive,
      ].every((phase) => !phase.isNetworkStall),
      isTrue,
    );
    expect(
      PlaybackSession(PlaybackVideoId.parse('clip'), deliveryId, 7),
      session,
    );
    expect(
      PlaybackSession(PlaybackVideoId.parse('clip'), deliveryId, 7).hashCode,
      session.hashCode,
    );
    expect(PlaybackVideoId.parse('clip').hashCode, videoId.hashCode);
    expect(() => PlaybackVideoId.parse('  '), throwsFormatException);
    expect(() => PlaybackSession(videoId, deliveryId, 0), throwsArgumentError);
    expect(
      () => PlaybackMetrics(
        position: Duration.zero,
        bufferedExtent: Duration.zero,
        playbackRate: 0,
      ),
      throwsArgumentError,
    );
    expect(
      () => PlaybackMetrics(
        position: const Duration(seconds: 6),
        bufferedExtent: const Duration(seconds: 5),
        playbackRate: 1,
      ),
      throwsArgumentError,
    );
    expect(
      () => PlaybackMetrics(
        position: const Duration(milliseconds: -1),
        bufferedExtent: Duration.zero,
        playbackRate: 1,
      ),
      throwsArgumentError,
    );
  });
}
