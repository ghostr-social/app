import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('sends twenty filters in one Nostr REQ', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
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
    ).thenReturn(NdkResponse('query', const Stream<Nip01Event>.empty()));
    final client = NdkNostrEventClient(
      ndk: ndk,
      relays: <RelayUrl>[RelayUrl.parse('wss://relay.example')],
    );
    final queries = <NostrEventQuery>[
      for (var index = 0; index < 20; index += 1)
        NostrEventQuery(kinds: <int>[index + 1]),
    ];

    expect(await client.queryBatch(queries), isEmpty);

    final call = verify(
      () => requests.query(
        name: 'ghostr-event-batch-query',
        // NDK 0.8.3 exposes multi-filter REQ only through this API.
        // ignore: deprecated_member_use
        filters: captureAny(named: 'filters'),
        explicitRelays: <String>['wss://relay.example'],
        timeout: const Duration(seconds: 5),
      ),
    );
    expect(call.captured.single as List<Filter>, hasLength(20));
  });
}
