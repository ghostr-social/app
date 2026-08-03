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

  test('rejects a signer that differs from the expected author', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    final broadcast = MockBroadcast();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(() => accounts.getLoggedAccount()).thenReturn(Account(
      type: AccountType.externalSigner,
      pubkey: testAuthorPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testAuthorPublicKey);
    when(() => signer.sign(any())).thenAnswer((call) async {
      return (call.positionalArguments.single as Nip01Event).copyWith(sig: 's');
    });
    when(() => cache.saveEvent(any())).thenAnswer((_) async {});
    when(
      () => broadcast.broadcast(
        nostrEvent: any(named: 'nostrEvent'),
        specificRelays: any(named: 'specificRelays'),
        saveToCache: false,
      ),
    ).thenAnswer((call) {
      final event = call.namedArguments[#nostrEvent] as Nip01Event;
      return NdkBroadcastResponse(
        publishEvent: event,
        broadcastDoneStream: Stream.value([
          RelayBroadcastResponse(
            relayUrl: 'wss://relay.example',
            broadcastSuccessful: true,
          ),
        ]),
      );
    });
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('account changed'),
      )),
    );

    verifyNever(() => signer.sign(any()));
    verifyNever(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          saveToCache: any(named: 'saveToCache'),
        ));
    verifyNever(() => cache.saveEvent(any()));
  });
}
