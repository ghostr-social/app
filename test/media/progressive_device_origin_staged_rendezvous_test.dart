import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('staged observation does not hold capacity before activation', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    final rendezvous = origin.stageFirstChunks({
      '/next.mp4',
      '/third.mp4',
    }, timeout: const Duration(milliseconds: 100));
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final first = _range(client, origin, 'next');
    await rendezvous.firstArrival.timeout(const Duration(seconds: 1));
    await first.timeout(const Duration(seconds: 1));
    await Future<void>.delayed(const Duration(milliseconds: 150));

    expect(rendezvous.timedOut, isFalse);
    expect(rendezvous.isReleased, isFalse);
    rendezvous.activate();
    final second = _range(client, origin, 'third');
    await rendezvous.reached.timeout(const Duration(seconds: 1));

    expect(rendezvous.arrivedPaths, {'/next.mp4', '/third.mp4'});
    await second;
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
