import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('device origin counts flushed bytes from a canceled range', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      responseChunkBytes: 1024,
      responseChunkDelay: const Duration(milliseconds: 20),
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });
    final request = await client.getUrl(origin.urlFor('current'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
    final response = await request.close();
    final firstChunk = Completer<void>();
    late final StreamSubscription<List<int>> subscription;
    subscription = response.listen((_) {
      if (!firstChunk.isCompleted) firstChunk.complete();
    });

    await firstChunk.future;
    await subscription.cancel();
    client.close(force: true);
    await Future<void>.delayed(const Duration(milliseconds: 120));

    expect(origin.bytesServed('current'), greaterThan(0));
    expect(origin.bytesServed('current'), lessThan(64 * 1024));
    expect(origin.rangesFor('current'), isEmpty);
  });
}
