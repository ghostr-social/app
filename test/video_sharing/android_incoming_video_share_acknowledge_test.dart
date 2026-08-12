import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('acknowledges an imported video through the Android gateway', () async {
    final gateway = FakeIncomingVideoShareGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    final media = SelectedMedia(
      path: '/cache/incoming-video-owned.mp4',
      source: MediaPickSource.externalShare,
      label: 'owned.mp4',
      mimeType: VideoMimeType.fromFileName('owned.mp4'),
    );

    await port.acknowledge(media);

    expect(gateway.acknowledgedPaths, [media.path]);
  });
}
