import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

import '../../integration_test/support/device_playback_probe.dart';

void main() {
  test('repeated focus evidence binds progress to its presented session', () {
    final probe = DevicePlaybackProbe();
    final video = PlaybackVideoId.parse('repeated');
    final old = probe.openSession(video, PlaybackDeliveryId.parse('old'));
    probe.markFocus(video);
    probe.activate(old);
    probe.presented(old);
    probe.report(_playing(old, 1));
    probe.markFocus(PlaybackVideoId.parse('between'));
    probe.deactivate(old);
    final repeated = probe.markFocus(video);
    probe.report(_playing(old, 9));
    probe.presented(old);
    expect(probe.sessionFor(repeated), isNull);
    final fresh = probe.openSession(video, PlaybackDeliveryId.parse('fresh'));
    probe.activate(fresh);
    probe.report(_playing(fresh, 2));
    probe.presented(fresh);

    expect(probe.sessionFor(repeated), fresh);
    expect(
      probe.phaseFor(repeated, PlaybackPhase.playing)?.observation.session,
      fresh,
    );
    expect(probe.latestPositionFor(repeated), const Duration(seconds: 2));
    probe.deactivate(fresh);
    probe.report(_playing(fresh, 12));
    expect(probe.latestPositionFor(repeated), const Duration(seconds: 2));
  });

  test('the next focus fences a late frame from its predecessor', () {
    final probe = DevicePlaybackProbe();
    final firstVideo = PlaybackVideoId.parse('first');
    final first = probe.markFocus(firstVideo);
    final session = probe.openSession(
      firstVideo,
      PlaybackDeliveryId.parse('first-delivery'),
    );
    probe.activate(session);

    probe.markFocus(PlaybackVideoId.parse('second'));
    probe.presented(session);

    expect(probe.sessionFor(first), isNull);
    expect(probe.firstFrameLatency(first), isNull);
  });
}

PlaybackObservation _playing(PlaybackSession session, int seconds) {
  return PlaybackObservation(
    session: session,
    phase: PlaybackPhase.playing,
    metrics: PlaybackMetrics(
      position: Duration(seconds: seconds),
      bufferedExtent: Duration(seconds: seconds + 2),
      playbackRate: 1,
    ),
  );
}
