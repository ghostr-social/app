import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/signed_event_fixture.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('fails the write when no relay accepts the event', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(() => response.broadcastDoneFuture).thenAnswer((_) async => []);
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    final adapter = NdkBroadcastAdapter(ndk: ndk, relays: const []);

    await expectLater(
      adapter.broadcast(encodeSignedNostrEvent(signedTestEvent())),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'No Nostr relay accepted the event.',
      )),
    );
  });
}
