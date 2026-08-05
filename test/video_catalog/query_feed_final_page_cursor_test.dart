import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('a final page without a cursor still leaves the feed a way forward',
      () async {
    final published = DateTime.utc(2026, 6, 15);
    final search = PagedSearchRepository(pages: [
      [
        samplePost(id: 'newer', publishedAt: DateTime.utc(2026, 6, 20)),
        samplePost(id: 'last', publishedAt: published),
      ],
    ]);
    final repository = QueryVideoFeedRepository(search: search, query: 'ghost');

    final page = await repository.loadOlderFeed(
      FeedKind.forYou,
      olderThan: DateTime.utc(2026, 7, 1),
    );

    expect(page.posts.map((post) => post.id.value), ['newer', 'last']);
    expect(page.hasMore, isTrue);
    expect(page.nextOlderThan, published.subtract(const Duration(seconds: 1)));
    expect(search.queries, isEmpty);
    expect(search.loadMoreQueries, ['ghost']);
  });
}
