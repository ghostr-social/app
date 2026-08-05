import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';

import '../support/paged_search_repository.dart';

void main() {
  test('a dry native page keeps pagination available without a head query',
      () async {
    final search = PagedSearchRepository(pages: [
      const [],
    ]);
    final repository = QueryVideoFeedRepository(search: search, query: 'ghost');
    final cursor = DateTime.utc(2026, 7, 1);

    final page = await repository.loadOlderFeed(
      FeedKind.forYou,
      olderThan: cursor,
    );

    expect(page.posts, isEmpty);
    expect(page.nextOlderThan, cursor);
    expect(page.hasMore, isTrue);
    expect(search.queries, isEmpty);
    expect(search.olderThans, isEmpty);
    expect(search.loadMoreQueries, ['ghost']);
  });
}
