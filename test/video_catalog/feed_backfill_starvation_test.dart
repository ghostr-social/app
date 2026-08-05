import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('the buffer starves once fewer than ten videos remain ahead', () {
    final repository = FakeVideoCatalogRepository(forYouFeed: const []);
    final backfill = FeedBackfill(FeedFetcher(repository), FeedLoads());
    final posts = [
      for (var index = 0; index < 20; index += 1) samplePost(id: 'post-$index'),
    ];

    expect(backfill.isStarved(FeedRoster(posts, activeIndex: 9)), isFalse);
    expect(backfill.isStarved(FeedRoster(posts, activeIndex: 10)), isTrue);
  });
}
