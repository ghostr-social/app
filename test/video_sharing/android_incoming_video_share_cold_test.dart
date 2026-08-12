import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('maps a pending supported video from a cold Android launch', () async {
    final gateway = FakeIncomingVideoShareGateway(
      pendingVideos: const [
        {
          'path': '/cache/shared/whatsapp-video.mp4',
          'label': 'whatsapp-video.mp4',
          'mimeType': 'video/mp4',
        },
      ],
    );
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);

    final event = await port.events.first;

    final ready = event as IncomingVideoShareReady;
    expect(ready.media.path, '/cache/shared/whatsapp-video.mp4');
    expect(ready.media.label, 'whatsapp-video.mp4');
    expect(ready.media.mimeType.value, 'video/mp4');
    expect(ready.media.source, MediaPickSource.externalShare);
    expect(gateway.takePendingVideoCalls, 1);
  });
}
