import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_mapper.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/signed_event_fixture.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('signs locally and sends canonical JSON only through the port',
      () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final signer = MockEventSigner();
    final port = RecordingSignedEventBroadcastPort();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.canSign).thenReturn(true);
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(() => signer.sign(any())).thenAnswer((call) async {
      final unsigned = call.positionalArguments.single as Nip01Event;
      return unsigned.copyWith(sig: testEventSignature);
    });
    final client = RustNostrEventClient(
      ndk: ndk,
      broadcast: port,
      mapper: RustNostrEventMapper(
        clock: () => DateTime.fromMillisecondsSinceEpoch(123000),
      ),
    );

    final id = await client.publish(
      NostrUnsignedEvent(
        kind: 7,
        tags: const <List<String>>[
          <String>['e', testEventId],
        ],
        content: '+',
      ),
      expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
    );

    final signed = decodeSignedNostrEvent(port.payloads.single);
    expect(id.value, signed.id);
    expect(signed.pubKey, testViewerPublicKey);
    expect(signed.createdAt, 123);
    expect(signed.kind, 7);
    expect(signed.tags.single, <String>['e', testEventId]);
    expect(signed.content, '+');
    expect(signed.sig, testEventSignature);
    verifyNever(() => ndk.requests);
    verifyNever(() => ndk.broadcast);
  });
}
