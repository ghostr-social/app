import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_relay_list_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test(
    'the newest relay-list event replaces every older declaration',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      client.events.addAll([
        relayListEvent(
          id: testEventId,
          createdAt: 10,
          tags: const [
            ['r', 'wss://old.example'],
          ],
        ),
        relayListEvent(
          id: secondTestEventId,
          createdAt: 20,
          tags: const [
            ['r', 'wss://new.example'],
          ],
        ),
      ]);

      final hint = await NostrAuthorWriteRelayLookup(client)(
        NostrPublicKeyHex.parse(testCreatorPublicKey),
      );

      expect(hint, 'wss://new.example');
    },
  );
}
