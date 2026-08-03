import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

class _GrowingFeedRepository extends FakeVideoCatalogRepository {
  _GrowingFeedRepository({required this.fresh, required this.refreshed})
      : super(forYouFeed: fresh);

  final List<VideoPost> fresh;
  final List<VideoPost> refreshed;
  var _loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    _loads += 1;
    return _loads == 1 ? fresh : refreshed;
  }
}

void main() {
  test('a refresh never inserts posts above the viewer position', () async {
    final watched = samplePost(id: 'watched-1');
    final current = samplePost(id: 'current-1');
    final next = samplePost(id: 'next-1');
    final repository = _GrowingFeedRepository(
      fresh: [current, next],
      refreshed: [watched, current, next],
    );
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);
    await cubit.load();
    cubit.pageChanged(0);

    await cubit.refresh();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['current-1', 'next-1']);
    expect(loaded.posts[loaded.activeIndex].id.value, 'current-1');
  });
}
