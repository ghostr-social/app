import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/video_discovery_queries.dart';

void main() {
  test('a search builds the video-kinds query plus a kind-1 note query', () {
    final queries = videoDiscoveryQueries(
      searchQuery: 'ghost dance',
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(queries, hasLength(2));
    final kinds = queries.map(
      (query) => query.kinds.map((kind) => kind.value).toList(),
    );
    expect(kinds.first, [21, 22, 34235, 34236]);
    expect(kinds.last, [1]);
    for (final query in queries) {
      expect(query.search, 'ghost dance');
      expect(query.limit, 200);
      expect(query.until, DateTime.utc(2026, 8, 1).millisecondsSinceEpoch ~/ 1000);
    }
  });

  test('a hashtag request fans the tag variants over both queries', () {
    final queries = videoDiscoveryQueries(hashtags: {'Dance'});

    expect(queries, hasLength(2));
    for (final query in queries) {
      expect(query.search, isNull);
      expect(query.tagFilters.single.name, 't');
      expect(
        query.tagFilters.single.values.toSet(),
        {'Dance', 'dance', 'DANCE'},
      );
    }
  });

  test('a plain feed request stays lean: one video query, no notes', () {
    final queries = videoDiscoveryQueries();

    expect(queries, hasLength(1));
    expect(queries.single.search, isNull);
    expect(queries.single.limit, 80);
    expect(queries.single.tagFilters, isEmpty);
  });
}
