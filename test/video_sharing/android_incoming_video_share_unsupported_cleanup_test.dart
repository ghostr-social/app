import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('releases a copied file when its MIME type is unsupported', () async {
    const path = '/cache/shared/unsupported.avi';
    final gateway = FakeIncomingVideoShareGateway(
      pendingVideos: [
        const {
          'path': path,
          'label': 'unsupported.avi',
          'mimeType': 'video/avi',
        },
      ],
    );
    final port = AndroidIncomingVideoSharePort(gateway);

    final event = await port.events.first;

    expect(event, isA<IncomingVideoShareFailure>());
    expect(gateway.releasedPaths, [path]);
    await gateway.close();
  });
}
