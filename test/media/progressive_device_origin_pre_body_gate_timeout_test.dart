import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'pre-body gate timeout starts when a matching request arrives',
    () async {
      const holdTimeout = Duration(milliseconds: 80);
      final origin = await ProgressiveDeviceOrigin.start();
      final client = HttpClient();
      final gate = origin.holdBeforeFirstBody({
        '/next.mp4',
      }, timeout: holdTimeout);
      addTearDown(() async {
        client.close(force: true);
        await origin.close();
      });

      await Future<void>.delayed(holdTimeout * 2);
      expect(gate.timedOut, isFalse);
      expect(gate.isReleased, isFalse);

      final transfer = _range(client, origin, 'next');
      await gate.reached.timeout(const Duration(seconds: 1));
      expect(gate.isReleased, isFalse);

      await transfer.timeout(const Duration(seconds: 1));
      expect(gate.timedOut, isTrue);
      expect(gate.isReleased, isTrue);
    },
  );
}

Future<void> _range(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
  await (await request.close()).drain<void>();
}
