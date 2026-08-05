import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('text creator search sends one trimmed NIP-50 kind-0 query', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final source = NostrCreatorSearchSource(client);

    await source.searchCreators('  Alice Relay  ');

    final query = client.queries.single;
    expect(query.kinds.map((kind) => kind.value), [0]);
    expect(query.search, 'Alice Relay');
    expect(query.limit, 30);
    expect(query.authors, isEmpty);
  });
}
