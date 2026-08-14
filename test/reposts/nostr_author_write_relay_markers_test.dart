import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_relay_list_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('a duplicate read tag cannot cancel writable relay evidence', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(
      relayListEvent(
        id: testEventId,
        createdAt: 10,
        tags: const [
          ['r', 'wss://read.example'],
          ['r', 'wss://read.example', 'read'],
          ['r', 'wss://unknown.example', 'private'],
          ['r', 'wss://chosen.example', 'write'],
          ['r', 'wss://later.example'],
        ],
      ),
    );

    final hint = await NostrAuthorWriteRelayLookup(client)(
      NostrPublicKeyHex.parse(testCreatorPublicKey),
    );

    expect(hint, 'wss://read.example');
  });
}
