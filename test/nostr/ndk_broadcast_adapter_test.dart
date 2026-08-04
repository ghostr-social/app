import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/signed_event_fixture.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('publishes the signed event unchanged to the configured relays',
      () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final signed = signedTestEvent(tags: [
      ['p', testAuthorPublicKey, 'wss://relay.example', 'friend'],
    ]);
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    final adapter = NdkBroadcastAdapter(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );

    await adapter.broadcast(encodeSignedNostrEvent(signed));

    final published = verify(() => broadcast.broadcast(
          nostrEvent: captureAny(named: 'nostrEvent'),
          specificRelays: ['wss://relay.example'],
          customSigner: signer,
          saveToCache: false,
        )).captured.single as Nip01Event;
    expect(published.id, signed.id);
    expect(published.pubKey, signed.pubKey);
    expect(published.createdAt, signed.createdAt);
    expect(published.kind, signed.kind);
    expect(published.tags, signed.tags);
    expect(published.content, signed.content);
    expect(published.sig, testEventSignature);
  });
}
