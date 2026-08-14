import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_relay_list_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('unsafe relay-list URLs are never emitted as repost hints', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(
      relayListEvent(
        id: testEventId,
        createdAt: 10,
        tags: const [
          ['r', 'ws://localhost:7777'],
          ['r', 'wss://127.0.0.1'],
          ['r', 'wss://[::ffff:127.0.0.1]'],
          ['r', 'wss://host.internal'],
          ['r', 'wss://relay.example.'],
          ['r', 'https://relay.example'],
        ],
      ),
    );

    final hint = await NostrAuthorWriteRelayLookup(client)(
      NostrPublicKeyHex.parse(testCreatorPublicKey),
    );

    expect(hint, isNull);
  });
}
