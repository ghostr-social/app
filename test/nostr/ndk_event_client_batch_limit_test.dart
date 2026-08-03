import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('preserves order while applying unique per-filter limits', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final first = _event(1);
    final targeted = _event(3, targeted: true);
    final returned = <Nip01Event>[
      first,
      first,
      _event(2),
      targeted,
      targeted,
      _event(4, targeted: true),
      _event(5, targeted: true),
    ];
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        // NDK 0.8.3 has no non-deprecated multi-filter REQ alternative.
        // ignore: deprecated_member_use
        filters: any(named: 'filters'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('batch', Stream.fromIterable(returned)));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    final result = await client.queryBatch(<NostrEventQuery>[
      NostrEventQuery(kinds: const <int>[7], limit: 1),
      NostrEventQuery(
        kinds: const <int>[7],
        scope: NostrEventQueryScope.parse(
          eventTags: const <String>[testEventId],
        ),
        limit: 2,
      ),
    ]);

    expect(result.map((event) => event.id.value), <String>[
      publishedEventId(1),
      publishedEventId(3),
      publishedEventId(4),
    ]);
  });
}

Nip01Event _event(int sequence, {bool targeted = false}) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: testAuthorPublicKey,
    kind: 7,
    tags: targeted
        ? const <List<String>>[
            <String>['e', testEventId],
          ]
        : const <List<String>>[],
    content: '',
    createdAt: sequence,
  );
}
