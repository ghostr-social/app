import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('an empty visible page preserves its advancing cursor', () async {
    final anchor = samplePost(id: 'anchor');
    final firstCursor = anchor.publishedAt.subtract(const Duration(seconds: 1));
    final secondCursor = firstCursor.subtract(const Duration(days: 1));
    final repository = _CursorFeed([
      VideoFeedPage(posts: const [], nextOlderThan: secondCursor),
      VideoFeedPage(posts: [samplePost(id: 'fresh')]),
    ]);
    final backfill = FeedBackfill(FeedFetcher(repository), FeedLoads());
    backfill.restartFrom([anchor]);

    final dry = await backfill.dig(FeedKind.forYou) as FeedDigPage;
    final fresh = await backfill.dig(FeedKind.forYou) as FeedDigPage;

    expect(dry.posts, isEmpty);
    expect(dry.hasMore, isTrue);
    expect(fresh.posts.single.id.value, 'fresh');
    expect(repository.requests, [firstCursor, secondCursor]);
  });
}

final class _CursorFeed implements VideoFeedRepository {
  _CursorFeed(this.pages);

  final List<VideoFeedPage> pages;
  final requests = <DateTime>[];

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async => const [];

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    requests.add(olderThan);
    return pages.removeAt(0);
  }
}
