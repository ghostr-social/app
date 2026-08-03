import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/video_discovery_queries.dart';

List<List<int>> kindsOf(List<NostrEventQuery> queries) => [
      for (final query in queries)
        [for (final kind in query.kinds) kind.value],
    ];

void main() {
  test('a search pairs the term with note and video-file queries', () {
    final queries = videoDiscoveryQueries(
      searchQuery: 'ghost dance',
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(kindsOf(queries), [
      [21, 22, 34235, 34236],
      [1],
      [1063],
    ]);
    final cutoff = DateTime.utc(2026, 8, 1).millisecondsSinceEpoch ~/ 1000;
    for (final query in queries) {
      expect(query.search, 'ghost dance');
      expect(query.limit, 200);
      expect(query.until, cutoff);
    }
    final mimes = queries.last.tagFilters.single;
    expect(mimes.name, 'm');
    expect(mimes.values, contains('video/mp4'));
  });

  test('a hashtag request fans the tag over notes, files, and an mp4 hunt',
      () {
    final queries = videoDiscoveryQueries(hashtags: {'Dance'});

    expect(kindsOf(queries), [
      [21, 22, 34235, 34236],
      [1],
      [1],
      [1063],
    ]);
    for (final query in queries) {
      final tags = query.tagFilters.firstWhere((filter) => filter.name == 't');
      expect(tags.values.toSet(), {'Dance', 'dance', 'DANCE'});
    }
    expect(queries[2].search, 'mp4');
    expect(
      queries.last.tagFilters.map((filter) => filter.name).toSet(),
      {'m', 't'},
    );
  });

  test('a plain feed hunts notes, mp4 mentions, and video files', () {
    final queries = videoDiscoveryQueries();

    expect(kindsOf(queries), [
      [21, 22, 34235, 34236],
      [1],
      [1],
      [1063],
    ]);
    expect(queries.first.limit, 80);
    expect(queries[1].search, isNull);
    expect(queries[1].limit, 200);
    expect(queries[2].search, 'mp4');
    expect(queries.last.tagFilters.single.name, 'm');
  });
}
