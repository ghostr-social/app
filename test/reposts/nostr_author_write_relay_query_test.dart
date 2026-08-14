import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_relay_list_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test(
    'queries the original author relay list and returns its write URL',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      client.events.add(
        relayListEvent(
          id: testEventId,
          createdAt: 10,
          tags: const [
            ['r', 'wss://write.example', 'write'],
          ],
        ),
      );

      final hint = await NostrAuthorWriteRelayLookup(client)(
        NostrPublicKeyHex.parse(testCreatorPublicKey),
      );

      final query = client.queries.single;
      expect(query.kinds.single.value, 10002);
      expect(query.authors.single.value, testCreatorPublicKey);
      expect(query.limit, 32);
      expect(hint, 'wss://write.example');
    },
  );
}
