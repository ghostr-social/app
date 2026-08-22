import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';

import '../../integration_test/support/device_playback_probe.dart';

void main() {
  test('device probe retains explicit presented-frame evidence', () {
    final probe = DevicePlaybackProbe();
    final session = probe.openSession(
      PlaybackVideoId.parse('video'),
      PlaybackDeliveryId.parse('delivery'),
    );

    probe.presented(session);

    expect(probe.presentations, [session]);
  });
}
