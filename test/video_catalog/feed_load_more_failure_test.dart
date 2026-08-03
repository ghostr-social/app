import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a failed older page keeps the feed and allows a retry', () async {
    final repository = _FlakyOlderFeedRepository([
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(9);
    await pumpEventQueue();
    final failed = cubit.state as FeedLoaded;
    expect(failed.posts, hasLength(12));
    expect(failed.notice, 'Older videos are unavailable right now.');

    repository.failOlderFeed = false;
    repository.olderFeedPages.add([samplePost(id: 'older-0')]);
    await cubit.loadMore();

    expect((cubit.state as FeedLoaded).posts, hasLength(13));
  });
}

class _FlakyOlderFeedRepository extends FakeVideoCatalogRepository {
  _FlakyOlderFeedRepository(List<VideoPost> posts) : super(forYouFeed: posts);

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
