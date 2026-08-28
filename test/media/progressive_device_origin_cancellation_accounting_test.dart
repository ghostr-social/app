import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('canceled response releases a shared link with exact bytes', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      responseChunkBytes: 1024,
      pacing: const ProgressiveOriginPacing.sharedBandwidth(2048),
    );
    final canceledClient = HttpClient();
    final peerClient = HttpClient();
    addTearDown(() async {
      canceledClient.close(force: true);
      peerClient.close(force: true);
      await origin.close();
    });
    final request = await canceledClient.getUrl(origin.urlFor('current'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
    final response = await request.close();
    final firstChunk = Completer<void>();
    late final StreamSubscription<List<int>> subscription;
    subscription = response.listen((_) {
      if (!firstChunk.isCompleted) firstChunk.complete();
    });

    await firstChunk.future;
    final peer = _readRange(peerClient, origin, 'next');
    await _waitForPeer(origin);
    await subscription.cancel();
    canceledClient.close(force: true);
    await peer.timeout(const Duration(seconds: 4));
    final recorded = origin.requestsFor('current').single;
    await _waitForTerminal(recorded);

    expect(origin.bytesServed('current'), greaterThan(0));
    expect(origin.bytesServed('current'), lessThan(64 * 1024));
    expect(origin.rangesFor('current'), isEmpty);
    expect(recorded.servedBytes, origin.bytesServed('current'));
    final window = origin.linkWindow(origin.currentLinkProfile!.generation);
    final currentEvents = window.events
        .where((event) => event.path == '/current.mp4')
        .toList();
    expect(
      currentEvents.every((event) => event.confirmedAtEpochMs != null),
      isTrue,
    );
    expect(
      currentEvents.fold(0, (bytes, event) => bytes + event.bytes),
      recorded.servedBytes,
    );
    expect(recorded.outcome, ProgressiveOriginRequestOutcome.clientCanceled);
    expect(recorded.finishedAt, isNotNull);
    expect(origin.coverageFor('next').isExact, isTrue);
  });
}

Future<void> _readRange(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  await (await request.close()).drain<void>();
}

Future<void> _waitForTerminal(ProgressiveOriginRequest request) async {
  for (var attempt = 0; attempt < 40; attempt += 1) {
    if (request.outcome != ProgressiveOriginRequestOutcome.serving) return;
    await Future<void>.delayed(const Duration(milliseconds: 50));
  }
  fail('Canceled origin request never became terminal.');
}

Future<void> _waitForPeer(ProgressiveDeviceOrigin origin) async {
  for (var attempt = 0; attempt < 40; attempt += 1) {
    if (origin.requestsFor('next').isNotEmpty) return;
    await Future<void>.delayed(const Duration(milliseconds: 25));
  }
  fail('Peer request never became active.');
}
