import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/entities.dart' show RelayBroadcastResponse;
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a relay-rejected publish never becomes query-visible', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    final broadcast = MockBroadcast();
    final requests = MockRequests();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    final visible = <Nip01Event>[];
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.requests).thenReturn(requests);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => accounts.getLoggedAccount()).thenReturn(Account(
      type: AccountType.externalSigner,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signer.sign(any())).thenAnswer((call) async {
      return (call.positionalArguments.single as Nip01Event).copyWith(sig: 's');
    });
    when(
      () => broadcast.broadcast(
        nostrEvent: any(named: 'nostrEvent'),
        specificRelays: any(named: 'specificRelays'),
        saveToCache: any(named: 'saveToCache'),
      ),
    ).thenAnswer((call) {
      final event = call.namedArguments[#nostrEvent] as Nip01Event;
      if (call.namedArguments[#saveToCache] != false) visible.add(event);
      return NdkBroadcastResponse(
        publishEvent: event,
        broadcastDoneStream: Stream.value(<RelayBroadcastResponse>[
          RelayBroadcastResponse(relayUrl: 'wss://relay.example'),
        ]),
      );
    });
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer((_) => NdkResponse('query', Stream.fromIterable(visible)));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(isA<AppFailure>()),
    );
    expect(await client.query(NostrEventQuery(kinds: const <int>[7])), isEmpty);
    verifyNever(() => cache.saveEvent(any()));
    verify(
      () => broadcast.broadcast(
        nostrEvent: any(named: 'nostrEvent'),
        specificRelays: any(named: 'specificRelays'),
        saveToCache: false,
      ),
    ).called(1);
  });
}
