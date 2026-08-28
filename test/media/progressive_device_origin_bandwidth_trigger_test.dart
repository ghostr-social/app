import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'bandwidth trigger changes an active response without blocking',
    () async {
      final origin = await ProgressiveDeviceOrigin.start(
        pacing: const ProgressiveOriginPacing.sharedBandwidth(100000),
      );
      final client = HttpClient();
      final trigger = origin.armBandwidthChangeAfterNextConfirmedChunk({
        '/next.mp4',
      }, bandwidthKbps: 1000);
      addTearDown(() async {
        client.close(force: true);
        await origin.close();
      });

      await _range(client, origin, 'next');
      await trigger.reached;
      expect(trigger.timedOut, isFalse);
      final profile = trigger.profile!;
      final sequence = trigger.requestSequence!;

      expect(trigger.path, '/next.mp4');
      expect(trigger.requestRange, (start: 0, end: 65536));
      expect(trigger.confirmedEvent?.profileGeneration, 1);
      expect(profile.activeRequestSequences, contains(sequence));
      expect(origin.requestSpansProfiles(sequence, {1, 2}), isTrue);
      final events = origin.confirmedChunkEventsFor('next');
      expect(events.first.profileGeneration, 1);
      expect(
        events.skip(1).every((event) => event.profileGeneration == 2),
        isTrue,
      );
      expect(origin.coverageFor('next').duplicateBytes, 0);
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
