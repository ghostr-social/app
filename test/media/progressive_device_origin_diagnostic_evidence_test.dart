import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('origin diagnostics correlate requests with confirmed chunks', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      pacing: const ProgressiveOriginPacing.sharedBandwidth(100000),
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });
    final request = await client.getUrl(origin.urlFor('next'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-32767');
    await (await request.close()).drain<void>();
    final recorded = origin.requestsFor('next').single;
    final events = origin.confirmedChunkEventsFor('next');

    expect(origin.requestSequenceFor(recorded), 1);
    expect(events, hasLength(2));
    expect(events.every((event) => event.requestSequence == 1), isTrue);
    expect(events.fold(0, (sum, event) => sum + event.bytes), 32768);
  });
}
