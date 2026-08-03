import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

const _coordinate = '34235:$testAuthorPublicKey:clip';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('drops relay events outside every field of a single filter', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final matching = _event(1);
    final returned = <Nip01Event>[
      matching,
      _event(2, kind: 6),
      _event(3, author: testCreatorPublicKey),
      _event(4, tags: const <List<String>>[
        <String>['e', secondTestEventId],
        <String>['a', _coordinate],
      ]),
      _event(5, tags: const <List<String>>[
        <String>['e', testEventId],
        <String>['a', '34235:$testAuthorPublicKey:other'],
      ]),
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
    final client = NdkNostrEventClient(
      ndk: ndk,
      relays: <RelayUrl>[RelayUrl.parse('wss://relay.example')],
    );
    final query = NostrEventQuery(
      kinds: const <int>[7],
      scope: NostrEventQueryScope.parse(
        authors: const <String>[testAuthorPublicKey],
        eventTags: const <String>[testEventId],
      ),
      tagFilters: <NostrTagFilter>[
        NostrTagFilter(name: 'a', values: const <String>[_coordinate]),
      ],
    );

    final result = await client.query(query);

    expect(result.map((event) => event.id), <String>[matching.id]);
  });
}

Nip01Event _event(
  int sequence, {
  int kind = 7,
  String author = testAuthorPublicKey,
  List<List<String>> tags = const <List<String>>[
    <String>['e', testEventId],
    <String>['a', _coordinate],
  ],
}) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: author,
    kind: kind,
    tags: tags,
    content: '+',
    createdAt: 10,
  );
}
