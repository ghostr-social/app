import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('retains warm videos until the app starts listening', () async {
    final gateway = FakeIncomingVideoShareGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    gateway.addPendingVideo(_payload('first'));
    gateway.notifyVideoAvailable();
    gateway.addPendingVideo(_payload('second'));
    gateway.notifyVideoAvailable();
    await Future<void>.delayed(Duration.zero);

    final events = await port.events
        .take(2)
        .toList()
        .timeout(const Duration(milliseconds: 500));

    expect(
      events.map((event) => (event as IncomingVideoShareReady).media.label),
      ['first.mp4', 'second.mp4'],
    );
  });
}

Map<Object?, Object?> _payload(String name) => {
  'path': '/cache/shared/$name.mp4',
  'label': '$name.mp4',
  'mimeType': 'video/mp4',
};
