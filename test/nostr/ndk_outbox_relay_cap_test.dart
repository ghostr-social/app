import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ndk/entities.dart';

import '../support/ndk_mocks.dart';
import '../support/outbox_directory_harness.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a tighter data budget keeps only the busiest outbox relays',
      () async {
    final harness = OutboxDirectoryHarness(viewer: testViewerPublicKey);
    harness.stubContacts([testCreatorPublicKey, testAuthorPublicKey]);
    harness.stubRelayList(testCreatorPublicKey, {
      'wss://shared.example': ReadWriteMarker.writeOnly,
      'wss://alpha.example': ReadWriteMarker.readWrite,
      'wss://beta.example': ReadWriteMarker.readWrite,
    });
    harness.stubRelayList(testAuthorPublicKey, {
      'wss://shared.example': ReadWriteMarker.readWrite,
    });
    final directory = NdkNostrOutboxDirectory(
      harness.ndk,
      bootstrapRelays: [RelayUrl.parse('wss://boot.example')],
      maxOutboxRelays: 2,
    );

    final relays = await directory.discoveryRelayUrls();

    // Bootstrap plus the two most-used outbox relays; beta is dropped.
    expect(relays, [
      'wss://boot.example',
      'wss://shared.example',
      'wss://alpha.example',
    ]);
  });
}
