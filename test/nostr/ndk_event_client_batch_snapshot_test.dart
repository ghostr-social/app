import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('matches a batch against the filters sent before awaiting relays',
      () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final events = StreamController<Nip01Event>();
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        // NDK 0.8.3 exposes multi-filter REQ only through this API.
        // ignore: deprecated_member_use
        filters: any(named: 'filters'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('batch', events.stream));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);
    final queries = <NostrEventQuery>[
      NostrEventQuery(kinds: const <int>[7]),
    ];

    final pending = client.queryBatch(queries);
    queries
      ..clear()
      ..add(NostrEventQuery(kinds: const <int>[6]));
    events.add(Nip01Event(
      id: testEventId,
      pubKey: testAuthorPublicKey,
      kind: 7,
      tags: const <List<String>>[],
      content: '+',
      createdAt: 10,
    ));
    await events.close();

    expect((await pending).single.id, testEventId);
  });
}
