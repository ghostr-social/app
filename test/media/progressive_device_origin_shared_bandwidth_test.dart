import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('one paced link changes rate on an already active request', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      pacing: const ProgressiveOriginPacing.sharedBandwidth(100000),
    );
    final client = HttpClient();
    final gate = origin.holdAfterChunks({
      '/next.mp4',
      '/third.mp4',
    }, afterChunks: 2);
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final first = _range(client, origin, 'next');
    final second = _range(client, origin, 'third');
    await gate.reached.timeout(const Duration(seconds: 2));
    expect(gate.requestRange, (start: 0, end: 131072));
    final fast = origin.currentLinkProfile!;
    final slow = origin.setBandwidthKbps(1000);

    expect(slow.activeRequestSequences, contains(gate.requestSequence));
    expect(slow.activeRequestSequences.length, greaterThanOrEqualTo(2));
    await _waitForPeerWindow(origin, slow.generation);
    final peerWindow = origin.linkWindow(slow.generation);
    expect(gate.isReleased, isFalse);
    expect(
      peerWindow.events.every(
        (event) => event.requestSequence != gate.requestSequence,
      ),
      isTrue,
    );
    gate.release();
    await Future.wait([first, second]);
    final window = origin.linkWindow(slow.generation);

    expect(window.duration, greaterThan(const Duration(milliseconds: 50)));
    expect(window.achievedBandwidthKbps, lessThanOrEqualTo(1000));
    expect(
      window.events.every((event) => event.confirmedAtEpochMs != null),
      isTrue,
    );
    expect(
      origin.requestSpansProfiles(gate.requestSequence!, {
        fast.generation,
        slow.generation,
      }),
      isTrue,
    );
  });
}

Future<void> _waitForPeerWindow(
  ProgressiveDeviceOrigin origin,
  int generation,
) async {
  final timeout = Stopwatch()..start();
  while (origin.linkWindow(generation).duration <
      const Duration(milliseconds: 100)) {
    if (timeout.elapsed > const Duration(seconds: 2)) {
      fail('A parallel request did not produce a paced link window.');
    }
    await Future<void>.delayed(const Duration(milliseconds: 10));
  }
}

Future<void> _range(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-131071');
  await (await request.close()).drain<void>();
}
