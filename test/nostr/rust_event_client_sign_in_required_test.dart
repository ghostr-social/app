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
  test('requires a sign-capable local account before publishing', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final port = RecordingSignedEventBroadcastPort();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(null);
    final client = RustNostrEventClient(ndk: ndk, broadcast: port);

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: const [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(isA<AppFailure>()),
    );
    expect(port.payloads, isEmpty);
  });

  test('rejects an active account whose signer cannot sign', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.externalSigner,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(false);
    final client = RustNostrEventClient(
      ndk: ndk,
      broadcast: RecordingSignedEventBroadcastPort(),
    );

    await expectLater(
      client.publish(
        NostrUnsignedEvent(kind: 7, tags: const [], content: '+'),
        expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
      throwsA(isA<AppFailure>()),
    );
    verifyNever(signer.getPublicKey);
  });
}
