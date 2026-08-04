import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_loads.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('digging walks one page into the past until it runs dry', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: const [])
      ..olderFeedPages.add([samplePost(id: 'older-0')]);
    final backfill = FeedBackfill(FeedFetcher(repository), FeedLoads());
    backfill.restartFrom([samplePost(id: 'post-0')]);

    final dug = await backfill.dig(FeedKind.forYou);

    expect((dug as FeedDigPage).posts.single.id.value, 'older-0');
    expect(
      repository.olderFeedRequests.single,
      samplePost().publishedAt.subtract(const Duration(seconds: 1)),
    );
    expect(await backfill.dig(FeedKind.forYou), isA<FeedDigSkipped>());
    expect(repository.olderFeedRequests, hasLength(1));
  });
}
