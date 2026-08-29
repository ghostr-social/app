import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'two future ranges deliver a first chunk before either continues',
    () async {
      final origin = await ProgressiveDeviceOrigin.start();
      final client = HttpClient();
      addTearDown(() async {
        client.close(force: true);
        await origin.close();
      });
      final rendezvous = origin.rendezvousFirstChunks({
        '/next.mp4',
        '/third.mp4',
      });

      var firstCompleted = false;
      final first = _range(
        client,
        origin,
        'next',
      ).whenComplete(() => firstCompleted = true);
      await rendezvous.firstArrival.timeout(const Duration(seconds: 1));
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(firstCompleted, isFalse);

      final second = _range(client, origin, 'third');
      await rendezvous.reached.timeout(const Duration(seconds: 1));

      expect(rendezvous.arrivedPaths, {'/next.mp4', '/third.mp4'});
      expect(rendezvous.isReleased, isTrue);
      expect(origin.bytesServed('next'), greaterThan(0));
      expect(origin.bytesServed('third'), greaterThan(0));
      await Future.wait([first, second]);
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
  final response = await request.close();
  await response.drain<void>();
}
