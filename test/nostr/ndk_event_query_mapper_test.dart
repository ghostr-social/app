import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates a domain event query into an NDK filter', () {
    const mapper = NdkNostrEventMapper();
    final query = NostrEventQuery(
      kinds: <int>[7],
      scope: NostrEventQueryScope.parse(
        authors: <String>[testAuthorPublicKey],
        eventTags: <String>[secondTestEventId],
      ),
      tagFilters: <NostrTagFilter>[
        NostrTagFilter(name: 'E', values: <String>['root-1']),
      ],
      limit: 25,
    );

    final filter = mapper.toFilter(query);

    expect(filter.kinds, <int>[7]);
    expect(filter.authors, <String>[testAuthorPublicKey]);
    expect(filter.eTags, <String>[secondTestEventId]);
    expect(filter.tags?['#E'], <String>['root-1']);
    expect(filter.limit, 25);
  });
}
