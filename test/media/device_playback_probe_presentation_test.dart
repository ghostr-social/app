import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';

import '../../integration_test/support/device_playback_probe.dart';

void main() {
  test('device probe measures the first matching frame after focus', () async {
    final probe = DevicePlaybackProbe();
    final videoId = PlaybackVideoId.parse('video');
    final staleSession = probe.openSession(
      videoId,
      PlaybackDeliveryId.parse('stale-delivery'),
    );
    probe.activate(staleSession);
    probe.presented(staleSession);
    await Future<void>.delayed(const Duration(milliseconds: 1));
    final focus = probe.markFocus(videoId);
    probe.presented(staleSession);
    final failedSession = probe.openSession(
      videoId,
      PlaybackDeliveryId.parse('failed-delivery'),
    );
    probe.activate(failedSession);
    final otherSession = probe.openSession(
      PlaybackVideoId.parse('other-video'),
      PlaybackDeliveryId.parse('other-delivery'),
    );
    probe.activate(otherSession);
    probe.presented(otherSession);

    expect(probe.firstFrameLatency(focus), isNull);

    final session = probe.openSession(
      PlaybackVideoId.parse('video'),
      PlaybackDeliveryId.parse('delivery'),
    );
    probe.activate(session);
    probe.presented(session);

    expect(probe.firstFrameLatency(focus), isNotNull);
    expect(probe.presentationFor(focus)?.session, session);
    expect(probe.presentations, [
      staleSession,
      staleSession,
      otherSession,
      session,
    ]);
  });

  test('a superseded session cannot supply the focused first frame', () {
    final probe = DevicePlaybackProbe();
    final videoId = PlaybackVideoId.parse('video');
    final focus = probe.markFocus(videoId);
    final target = probe.openSession(
      videoId,
      PlaybackDeliveryId.parse('target'),
    );
    final replacement = probe.openSession(
      PlaybackVideoId.parse('replacement'),
      PlaybackDeliveryId.parse('replacement'),
    );

    probe.activate(target);
    probe.activate(replacement);
    probe.presented(target);

    expect(probe.firstFrameLatency(focus), isNull);
  });

  test('a deactivated session cannot supply the focused first frame', () {
    final probe = DevicePlaybackProbe();
    final videoId = PlaybackVideoId.parse('video');
    final focus = probe.markFocus(videoId);
    final session = probe.openSession(
      videoId,
      PlaybackDeliveryId.parse('delivery'),
    );

    probe.activate(session);
    probe.deactivate(session);
    probe.presented(session);

    expect(probe.firstFrameLatency(focus), isNull);
  });
}
