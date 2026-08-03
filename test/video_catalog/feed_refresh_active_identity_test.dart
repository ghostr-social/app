import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

class _ShrinkingFeedRepository extends FakeVideoCatalogRepository {
  _ShrinkingFeedRepository({required this.fresh, required this.refreshed})
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
  test('a refresh keeps the active video when posts above it disappear',
      () async {
    final blocked = samplePost(id: 'blocked-1');
    final active = samplePost(id: 'active-1');
    final repository = _ShrinkingFeedRepository(
      fresh: [blocked, active],
      refreshed: [active],
    );
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);
    await cubit.load();
    cubit.pageChanged(1);

    await cubit.refresh();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts[loaded.activeIndex].id.value, 'active-1');
  });
}
