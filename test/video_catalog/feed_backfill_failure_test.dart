import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a failed dig reports why and frees the next attempt', () async {
    final repository = _FlakyOlderFeedRepository();
    final backfill = FeedBackfill(FeedFetcher(repository), FeedLoads());
    backfill.restartFrom([samplePost(id: 'post-0')]);

    final failed = await backfill.dig(FeedKind.forYou);

    final cause = (failed as FeedDigFailed).failure.failure.cause;
    expect((cause as AppFailure).message,
        'Older videos are unavailable right now.');

    repository.failOlderFeed = false;
    repository.olderFeedPages.add([samplePost(id: 'older-0')]);
    final retried = await backfill.dig(FeedKind.forYou);

    expect((retried as FeedDigPage).posts.single.id.value, 'older-0');
  });
}

class _FlakyOlderFeedRepository extends FakeVideoCatalogRepository {
  _FlakyOlderFeedRepository() : super(forYouFeed: const []);

  bool failOlderFeed = true;

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    if (failOlderFeed) {
      throw const AppFailure('Older videos are unavailable right now.');
    }
    return super.loadOlderFeed(
      kind,
      olderThan: olderThan,
      excludeWatched: excludeWatched,
    );
  }
}
