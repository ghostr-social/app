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

  test('a plain feed request adds a wide kind-1 note query', () {
    final queries = videoDiscoveryQueries();

    expect(queries, hasLength(2));
    final video = queries.first;
    final notes = queries.last;
    expect(video.kinds.map((kind) => kind.value), [21, 22, 34235, 34236]);
    expect(video.limit, 80);
    expect(notes.kinds.map((kind) => kind.value), [1]);
    expect(notes.limit, 200);
    for (final query in queries) {
      expect(query.search, isNull);
      expect(query.tagFilters, isEmpty);
    }
  });
}
