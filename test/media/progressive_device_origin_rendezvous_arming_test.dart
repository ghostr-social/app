import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('rendezvous watchdog starts when its first response arrives', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    final rendezvous = origin.rendezvousFirstChunks({
      '/next.mp4',
      '/third.mp4',
    }, timeout: const Duration(milliseconds: 50));
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    await Future<void>.delayed(const Duration(milliseconds: 75));
    expect(rendezvous.timedOut, isFalse);
    final transfers = [
      _range(client, origin, 'next'),
      _range(client, origin, 'third'),
    ];
    await rendezvous.reached.timeout(const Duration(seconds: 1));

    expect(rendezvous.timedOut, isFalse);
    await Future.wait(transfers);
  });
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
