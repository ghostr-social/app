import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates a local signing failure at the client boundary', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signer.sign(any())).thenThrow(StateError('signer failed'));
    final client = RustNostrEventClient(
      ndk: ndk,
      broadcast: RecordingSignedEventBroadcastPort(),
    );

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: const [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not publish the Nostr event.',
        ),
      ),
    );
  });
}
