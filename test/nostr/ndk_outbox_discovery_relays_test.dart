import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/entities.dart';

import '../support/ndk_mocks.dart';
import '../support/outbox_directory_harness.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('discovery targets the relays most of your follows write to',
      () async {
    final harness = OutboxDirectoryHarness(viewer: testViewerPublicKey);
    harness.stubContacts([testCreatorPublicKey, testAuthorPublicKey]);
    harness.stubRelayList(testCreatorPublicKey, {
      'wss://alpha.example': ReadWriteMarker.readWrite,
      'wss://shared.example': ReadWriteMarker.writeOnly,
      'wss://inbox.example': ReadWriteMarker.readOnly,
    });
    harness.stubRelayList(testAuthorPublicKey, {
      'wss://shared.example': ReadWriteMarker.readWrite,
    });
    final directory = NdkNostrOutboxDirectory(
      harness.ndk,
      bootstrapRelays: [RelayUrl.parse('wss://boot.example')],
      clock: () => DateTime.utc(2026, 8, 3),
    );

    final relays = await directory.discoveryRelayUrls();

    expect(relays, [
      'wss://boot.example',
      'wss://shared.example',
      'wss://alpha.example',
    ]);

    await directory.discoveryRelayUrls();
    verify(() => harness.follows.getContactList(testViewerPublicKey))
        .called(1);
  });
}
