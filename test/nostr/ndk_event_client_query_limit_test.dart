import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('returns the first unique matches up to the filter limit', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final first = _event(1);
    final returned = <Nip01Event>[
      first,
      first,
      _event(2),
      _event(3),
    ];
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('query', Stream.fromIterable(returned)));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    final result = await client.query(
      NostrEventQuery(kinds: const <int>[7], limit: 2),
    );

    expect(result.map((event) => event.id.value), <String>[
      publishedEventId(1),
      publishedEventId(2),
    ]);
  });
}

Nip01Event _event(int sequence) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: testAuthorPublicKey,
    kind: 7,
    tags: const <List<String>>[],
    content: '+',
    createdAt: sequence,
  );
}
