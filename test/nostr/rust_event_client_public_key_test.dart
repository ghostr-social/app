import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('returns the typed active local Nostr public key', () {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    final client = _client(ndk);

    expect(client.publicKeyHex, testViewerPublicKey);
  });

  test('requires an active local Nostr account for its public key', () {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(null);
    final client = _client(ndk);

    expect(() => client.publicKeyHex, throwsA(isA<AppFailure>()));
  });
}

RustNostrEventClient _client(MockNdk ndk) {
  return RustNostrEventClient(
    ndk: ndk,
    broadcast: RecordingSignedEventBroadcastPort(),
  );
}
