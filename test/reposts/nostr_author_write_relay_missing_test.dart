import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('an author without a relay list has no repost hint', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);

    final hint = await NostrAuthorWriteRelayLookup(client)(
      NostrPublicKeyHex.parse(testCreatorPublicKey),
    );

    expect(hint, isNull);
  });
}
