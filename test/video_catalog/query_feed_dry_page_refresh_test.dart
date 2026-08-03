import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('a dry older page hunts for fresh matches and keeps the cursor',
      () async {
    final search = PagedSearchRepository(pages: [
      const [],
      [samplePost(id: 'fresh')],
    ]);
    final repository = QueryVideoFeedRepository(search: search, query: 'ghost');
    final cursor = DateTime.utc(2026, 7, 1);

    final page = await repository.loadOlderFeed(
      FeedKind.forYou,
      olderThan: cursor,
    );

    expect(page.posts.map((post) => post.id.value), ['fresh']);
    expect(page.nextOlderThan, cursor);
    expect(page.hasMore, isTrue);
    expect(search.queries, ['ghost', 'ghost']);
    expect(search.olderThans, [cursor, null]);
  });
}
