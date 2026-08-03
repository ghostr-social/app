import 'package:flutter_test/flutter_test.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('text search pages relay results newest-first without blocked creators',
      () async {
    final blocked = sampleCreator(id: 'npub-blocked');
    final harness = DiscoverySearchHarness(posts: [
      samplePost(id: 'old', publishedAt: DateTime.utc(2026, 1, 1)),
      samplePost(id: 'new', publishedAt: DateTime.utc(2026, 3, 1)),
      samplePost(
        id: 'hidden',
        creator: blocked,
        publishedAt: DateTime.utc(2025, 12, 25),
      ),
    ]);
    harness.social.blocked.add(blocked.id);
    final repository = harness.repository;

    final cursor = DateTime.utc(2026, 4, 1);
    final page = await repository.searchVideos(' Ghost ', olderThan: cursor);

    expect(harness.source.searchQueries, ['ghost']);
    expect(harness.source.hashtags, [null]);
    expect(harness.source.olderThans, [cursor]);
    expect(page.posts.map((post) => post.id.value), ['new', 'old']);
    // The cursor advances past the oldest fetched post, blocked or not.
    expect(
      page.nextOlderThan,
      DateTime.utc(2025, 12, 25).subtract(const Duration(seconds: 1)),
    );

    final blank = await repository.searchVideos('   ');
    expect(blank.posts, isEmpty);
    expect(blank.hasMore, isFalse);
    expect(harness.source.searchQueries, hasLength(1));
  });
}
