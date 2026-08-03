import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('keeps matches for any batch filter without mixing their fields',
      () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final reaction = _event(1, 7, <List<String>>[
      <String>['e', testEventId],
    ]);
    final comment = _event(2, 1111, <List<String>>[
      const <String>['A', '34235:author:clip'],
    ]);
    final crossed = _event(3, 7, <List<String>>[
      const <String>['A', '34235:author:clip'],
    ]);
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
    ).thenReturn(
      NdkResponse('batch', Stream.fromIterable([reaction, comment, crossed])),
    );
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);
    final queries = <NostrEventQuery>[
      NostrEventQuery(
        kinds: const <int>[7],
        scope: NostrEventQueryScope.parse(
          eventTags: const <String>[testEventId],
        ),
        limit: 1,
      ),
      NostrEventQuery(
        kinds: const <int>[1111],
        tagFilters: <NostrTagFilter>[
          NostrTagFilter(
            name: 'A',
            values: const <String>['34235:author:clip'],
          ),
        ],
        limit: 1,
      ),
    ];

    final result = await client.queryBatch(queries);

    expect(result.map((event) => event.id), <String>[reaction.id, comment.id]);
  });
}

Nip01Event _event(int sequence, int kind, List<List<String>> tags) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: testAuthorPublicKey,
    kind: kind,
    tags: tags,
    content: '+',
    createdAt: 10,
  );
}
