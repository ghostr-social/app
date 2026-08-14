import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_relay_list_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('equal relay-list timestamps choose the lowest event id', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.addAll([
      relayListEvent(
        id: secondTestEventId,
        createdAt: 10,
        tags: const [
          ['r', 'wss://higher-id.example'],
        ],
      ),
      relayListEvent(
        id: testEventId,
        createdAt: 10,
        tags: const [
          ['r', 'wss://lower-id.example'],
        ],
      ),
    ]);

    final hint = await NostrAuthorWriteRelayLookup(client)(
      NostrPublicKeyHex.parse(testCreatorPublicKey),
    );

    expect(hint, 'wss://lower-id.example');
  });
}
