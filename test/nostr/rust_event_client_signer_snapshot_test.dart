import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
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

  test('finishes with the signer captured before an account switch', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signerA = MockEventSigner();
    final signerB = MockEventSigner();
    final signStarted = Completer<void>();
    final releaseSign = Completer<void>();
    final accountA = _account(testViewerPublicKey, signerA);
    final accountB = _account(testAuthorPublicKey, signerB);
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(accountA);
    when(signerA.canSign).thenReturn(true);
    when(signerA.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signerA.sign(any())).thenAnswer((call) async {
      signStarted.complete();
      await releaseSign.future;
      final unsigned = call.positionalArguments.single as Nip01Event;
      return unsigned.copyWith(sig: testEventSignature);
    });
    final client = RustNostrEventClient(
      ndk: ndk,
      broadcast: RecordingSignedEventBroadcastPort(),
    );

    final pending = client.publish(
      NostrUnsignedEvent(kind: 7, tags: const [], content: '+'),
      expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
    );
    await signStarted.future;
    when(accounts.getLoggedAccount).thenReturn(accountB);
    releaseSign.complete();

    expect(await pending, isNotEmpty);
    verifyNever(() => signerB.sign(any()));
    verify(accounts.getLoggedAccount).called(1);
  });
}

Account _account(String publicKey, EventSigner signer) {
  return Account(
    type: AccountType.privateKey,
    pubkey: publicKey,
    signer: signer,
  );
}
