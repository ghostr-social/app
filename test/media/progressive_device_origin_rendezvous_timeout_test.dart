import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('first-chunk rendezvous fails open when a peer never arrives', () async {
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

    final transfer = _range(client, origin, 'next');
    await rendezvous.firstArrival.timeout(const Duration(seconds: 1));
    expect(rendezvous.arrivedPaths, {'/next.mp4'});
    await rendezvous.settled.timeout(const Duration(seconds: 1));

    expect(rendezvous.timedOut, isTrue);
    expect(rendezvous.isReleased, isTrue);
    await transfer;
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
