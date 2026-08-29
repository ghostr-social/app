import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('activated staged observation never holds the first response', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    final observation = origin.stageFirstChunks({'/next.mp4', '/third.mp4'});
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    observation.activate();
    await _range(
      client,
      origin,
      'next',
    ).timeout(const Duration(milliseconds: 500));

    expect(observation.arrivedPaths, {'/next.mp4'});
    expect(observation.isSettled, isFalse);
    await _range(client, origin, 'third');
    await observation.reached.timeout(const Duration(seconds: 1));
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
