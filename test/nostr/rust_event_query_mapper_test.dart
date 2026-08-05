import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_mapper.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('preserves every supported query field in the Rust filter', () {
    const mapper = RustNostrEventMapper();
    final query = NostrEventQuery(
      kinds: const <int>[7, 1111],
      scope: NostrEventQueryScope.parse(
        authors: const <String>[testAuthorPublicKey],
        eventTags: const <String>[secondTestEventId],
      ),
      tagFilters: <NostrTagFilter>[
        NostrTagFilter(name: 'A', values: const <String>['root']),
      ],
      limit: 25,
      until: 1700000000,
      search: 'ghost dance',
    );

    final filter = mapper.toFilter(query);

    expect(filter.kinds, <int>[7, 1111]);
    expect(filter.authors, <String>[testAuthorPublicKey]);
    expect(filter.eventTags, <String>[secondTestEventId]);
    expect(filter.tagFilters.single.name, 'A');
    expect(filter.tagFilters.single.values, <String>['root']);
    expect(filter.limit, 25);
    expect(filter.until, BigInt.from(1700000000));
    expect(filter.search, 'ghost dance');
  });
}
