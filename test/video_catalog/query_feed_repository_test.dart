import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('a query feed serves its bound search for every feed kind', () async {
    final search = PagedSearchRepository(pages: [
      [samplePost(id: 'first')],
      [samplePost(id: 'older')],
    ]);
    final repository =
        QueryVideoFeedRepository(search: search, query: '#dance');

    final initial = await repository.loadFeed(
      FeedKind.following,
      excludeWatched: true,
    );
    expect(initial.map((post) => post.id.value), ['first']);

    final cursor = DateTime.utc(2026, 2, 1);
    final older = await repository.loadOlderFeed(
      FeedKind.forYou,
      olderThan: cursor,
    );
    expect(older.posts.map((post) => post.id.value), ['older']);
    expect(search.queries, ['#dance']);
    expect(search.olderThans, [null]);
    expect(search.loadMoreQueries, ['#dance']);
  });
}
