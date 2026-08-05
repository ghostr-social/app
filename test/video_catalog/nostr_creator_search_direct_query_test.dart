import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('npub and hexadecimal creator ids use a one-author lookup', () async {
    for (final identifier in [testViewerNpub, testViewerPublicKey]) {
      final client = FakeNostrEventClient(publicKeyHex: testCreatorPublicKey);
      final source = NostrCreatorSearchSource(client);

      await source.searchCreators('  $identifier  ');

      final query = client.queries.single;
      expect(query.kinds.map((kind) => kind.value), [0]);
      expect(query.authors.map((author) => author.value), [
        testViewerPublicKey,
      ]);
      expect(query.limit, 1);
      expect(query.search, isNull);
    }
  });
}
