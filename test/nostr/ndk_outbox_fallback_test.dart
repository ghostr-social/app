import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/outbox_directory_harness.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('discovery falls back to bootstrap relays without outbox data',
      () async {
    final signedOut = OutboxDirectoryHarness(viewer: testViewerPublicKey);
    when(signedOut.accounts.getPublicKey).thenReturn(null);
    final directory = NdkNostrOutboxDirectory(
      signedOut.ndk,
      bootstrapRelays: [RelayUrl.parse('wss://boot.example')],
    );
    expect(await directory.discoveryRelayUrls(), ['wss://boot.example']);
    verifyNever(() => signedOut.follows.getContactList(any()));

    final failing = OutboxDirectoryHarness(viewer: testViewerPublicKey);
    when(() => failing.follows.getContactList(testViewerPublicKey))
        .thenThrow(StateError('relay down'));
    final failingDirectory = NdkNostrOutboxDirectory(
      failing.ndk,
      bootstrapRelays: [RelayUrl.parse('wss://boot.example')],
    );
    expect(
      await failingDirectory.discoveryRelayUrls(),
      ['wss://boot.example'],
    );
  });
}
