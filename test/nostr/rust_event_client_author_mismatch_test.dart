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

  test('rejects an active signer different from the expected author', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    final port = RecordingSignedEventBroadcastPort();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testAuthorPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testAuthorPublicKey);
    final client = RustNostrEventClient(ndk: ndk, broadcast: port);

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: const [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('account changed'),
        ),
      ),
    );

    verifyNever(() => signer.sign(any()));
    expect(port.payloads, isEmpty);
  });
}
