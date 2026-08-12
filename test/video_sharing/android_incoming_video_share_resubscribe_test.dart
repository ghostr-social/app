import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('checks initial native state only for the first listener', () async {
    final gateway = FakeIncomingVideoShareGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    final first = port.events.listen((_) {});
    await gateway.firstTake;
    await Future<void>.delayed(Duration.zero);
    await first.cancel();

    final second = port.events.listen((_) {});
    await Future<void>.delayed(Duration.zero);

    expect(gateway.takePendingVideoCalls, 1);
    await second.cancel();
  });
}
