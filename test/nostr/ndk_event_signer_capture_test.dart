import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
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

  test('finishes with the signer captured before an account switch', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signerA = MockEventSigner();
    final signerB = MockEventSigner();
    final broadcast = MockBroadcast();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    final signStarted = Completer<void>();
    final releaseSign = Completer<void>();
    final accountA = Account(
      type: AccountType.externalSigner,
      pubkey: testViewerPublicKey,
      signer: signerA,
    );
    final accountB = Account(
      type: AccountType.externalSigner,
      pubkey: testAuthorPublicKey,
      signer: signerB,
    );
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(() => accounts.getLoggedAccount()).thenReturn(accountA);
    when(signerA.canSign).thenReturn(true);
    when(signerA.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signerA.sign(any())).thenAnswer((call) async {
      signStarted.complete();
      await releaseSign.future;
      return (call.positionalArguments.single as Nip01Event).copyWith(sig: 'a');
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

    final pending = client.publish(
      NostrUnsignedEvent(kind: 7, tags: [], content: '+'),
      expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
    );
    await signStarted.future;
    when(() => accounts.getLoggedAccount()).thenReturn(accountB);
    releaseSign.complete();

    expect(await pending, isNotEmpty);
    verifyNever(() => signerB.sign(any()));
    verify(() => accounts.getLoggedAccount()).called(1);
  });
}
