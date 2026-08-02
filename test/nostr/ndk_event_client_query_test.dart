import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('queries configured relays and maps NDK events', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final event = Nip01Event(
      id: testEventId,
      pubKey: testAuthorPublicKey,
      kind: 7,
      tags: const [
        ['e', secondTestEventId],
      ],
      content: '+',
      createdAt: 10,
    );
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('query', Stream.value(event)));
    final client = NdkNostrEventClient(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );

    final records = await client.query(
      NostrEventQuery(
        kinds: [7],
        scope: NostrEventQueryScope.parse(eventTags: [secondTestEventId]),
      ),
    );

    expect(records.single.id, testEventId);
    final call = verify(
      () => requests.query(
        name: 'ghostr-event-query',
        filter: captureAny(named: 'filter'),
        explicitRelays: captureAny(named: 'explicitRelays'),
        timeout: const Duration(seconds: 5),
      ),
    );
    expect((call.captured[0] as Filter).kinds, [7]);
    expect(call.captured[1], ['wss://relay.example']);
  });
}
