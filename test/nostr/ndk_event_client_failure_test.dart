import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates low-level NDK query and publish failures', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    when(() => ndk.requests).thenReturn(requests);
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => accounts.getLoggedAccount()).thenReturn(Account(
      type: AccountType.externalSigner,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signer.sign(any())).thenThrow(StateError('signer failed'));
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenThrow(StateError('socket failed'));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    await expectLater(
      client.query(NostrEventQuery(kinds: [7])),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(isA<AppFailure>()),
    );
  });
}
