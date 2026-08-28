import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'one backpressured response cannot monopolize the shared link',
    () async {
      final origin = await ProgressiveDeviceOrigin.start(
        pacing: const ProgressiveOriginPacing.sharedBandwidth(10000),
      );
      final blockedClient = HttpClient();
      final peerClient = HttpClient();
      addTearDown(() async {
        blockedClient.close(force: true);
        peerClient.close(force: true);
        await origin.close();
      });

      final blocked = await blockedClient.getUrl(origin.urlFor('current'));
      final response = await blocked.close();
      final subscription = response.listen((_) {});
      subscription.pause();
      addTearDown(subscription.cancel);

      final peer = await peerClient.getUrl(origin.urlFor('next'));
      await (await peer.close()).drain<void>().timeout(
        const Duration(seconds: 2),
      );

      expect(origin.coverageFor('next').isExact, isTrue);
    },
  );
}
