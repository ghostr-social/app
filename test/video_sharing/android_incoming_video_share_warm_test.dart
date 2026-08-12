import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('drains a pending video after a warm Android notification', () async {
    final gateway = FakeIncomingVideoShareGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    final nextEvent = port.events.first;
    await gateway.firstTake;
    gateway.addPendingVideo(const {
      'path': '/cache/shared/warm-video.webm',
      'label': 'warm-video.webm',
      'mimeType': 'video/webm',
    });

    gateway.notifyVideoAvailable();
    final event = await nextEvent;

    final ready = event as IncomingVideoShareReady;
    expect(ready.media.path, '/cache/shared/warm-video.webm');
    expect(ready.media.mimeType.value, 'video/webm');
    expect(gateway.takePendingVideoCalls, 2);
  });
}
