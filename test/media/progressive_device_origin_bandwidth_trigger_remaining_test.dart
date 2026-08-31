import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'a short remaining response cannot consume a bandwidth trigger',
    () async {
      final origin = await ProgressiveDeviceOrigin.start(
        pacing: const ProgressiveOriginPacing.sharedBandwidth(100000),
      );
      final client = HttpClient();
      final trigger = origin.armBandwidthChangeAfterNextConfirmedChunk(
        {'/short.mp4', '/long.mp4'},
        bandwidthKbps: 1000,
        minimumRemainingBytes: 160 * 1024,
      );
      addTearDown(() async {
        client.close(force: true);
        await origin.close();
      });

      await _range(client, origin, 'short', 160 * 1024);
      expect(trigger.isReached, isFalse);
      await _range(client, origin, 'long', 192 * 1024);
      await trigger.reached;

      expect(trigger.timedOut, isFalse);
      expect(trigger.path, '/long.mp4');
      expect(trigger.profile?.activeRequestSequences, contains(2));
    },
  );
}

Future<void> _range(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
  int bytes,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-${bytes - 1}');
  await (await request.close()).drain<void>();
}
