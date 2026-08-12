import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'does not send local picker media to the incoming-share gateway',
    () async {
      final gateway = FakeIncomingVideoShareGateway();
      addTearDown(gateway.close);
      final port = AndroidIncomingVideoSharePort(gateway);
      final media = sampleMedia();

      await port.acknowledge(media);
      await port.release(media);

      expect(gateway.acknowledgedPaths, isEmpty);
      expect(gateway.releasedPaths, isEmpty);
    },
  );
}
