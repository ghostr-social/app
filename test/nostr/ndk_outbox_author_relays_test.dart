import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ndk/entities.dart';

import '../support/ndk_mocks.dart';
import '../support/outbox_directory_harness.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('author queries target the write relays those authors declared',
      () async {
    final harness = OutboxDirectoryHarness(viewer: testViewerPublicKey);
    harness.stubRelayList(testCreatorPublicKey, {
      'wss://creator.example/': ReadWriteMarker.readWrite,
    });
    harness.stubRelayList(testAuthorPublicKey, {
      'wss://author.example': ReadWriteMarker.writeOnly,
      'not a relay': ReadWriteMarker.readWrite,
    });
    final directory = NdkNostrOutboxDirectory(
      harness.ndk,
      bootstrapRelays: [RelayUrl.parse('wss://boot.example')],
    );

    final relays = await directory.authorWriteRelayUrls({
      NostrPublicKeyHex.parse(testCreatorPublicKey),
      NostrPublicKeyHex.parse(testAuthorPublicKey),
    });

    expect(relays.first, 'wss://boot.example');
    expect(
      relays.skip(1),
      unorderedEquals(
        <String>['wss://creator.example', 'wss://author.example'],
      ),
    );
  });
}
