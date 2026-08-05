import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/signed_event_fixture.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects a signed event whose author differs from the expected one',
      () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    final port = RecordingSignedEventBroadcastPort();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.externalSigner,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signer.sign(any())).thenAnswer((call) async {
      final unsigned = call.positionalArguments.single as Nip01Event;
      return unsigned.copyWith(
        pubKey: testAuthorPublicKey,
        sig: testEventSignature,
      );
    });
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
}
