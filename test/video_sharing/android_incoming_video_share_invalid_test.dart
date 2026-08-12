import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

const _safeMessage = 'Could not open the shared video.';

void main() {
  test('turns malformed and unsupported payloads into safe failures', () async {
    for (final payload in <Map<Object?, Object?>>[
      const {'path': 42, 'label': 'broken.mp4', 'mimeType': 'video/mp4'},
      const {
        'path': '/cache/shared/unsupported.avi',
        'label': 'unsupported.avi',
        'mimeType': 'video/avi',
      },
    ]) {
      final gateway = FakeIncomingVideoShareGateway(pendingVideos: [payload]);
      final port = AndroidIncomingVideoSharePort(gateway);

      final event = await port.events.first;

      final failure = event as IncomingVideoShareFailure;
      expect(failure.message, _safeMessage);
      await gateway.close();
    }
  });
}
