import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('requires an active Nostr account for its public key', () {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(null);
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    expect(() => client.publicKeyHex, throwsA(isA<AppFailure>()));
  });

  test('returns the typed active Nostr public key', () {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    expect(client.publicKeyHex, testViewerPublicKey);
  });
}
