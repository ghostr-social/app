import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test(
    'close detaches the app-lifetime native availability listener',
    () async {
      final gateway = FakeIncomingVideoShareGateway();
      final port = AndroidIncomingVideoSharePort(gateway);

      expect(gateway.hasVideoAvailableListener, isTrue);

      await port.close();

      expect(gateway.hasVideoAvailableListener, isFalse);
      expect(gateway.closeCalls, 1);
      await port.close();
      expect(gateway.closeCalls, 1);
    },
  );
}
