import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

class _FlakyFeedRepository extends FakeVideoCatalogRepository {
  _FlakyFeedRepository(List<VideoPost> posts) : super(forYouFeed: posts);

  var _loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    _loads += 1;
    if (_loads > 1) throw const AppFailure('No relay reachable.');
    return super.loadFeed(kind, excludeWatched: excludeWatched);
  }
}

void main() {
  test('a failed reload keeps the current feed and shows a notice', () async {
    final post = samplePost();
    final repository = _FlakyFeedRepository([post]);
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.reload();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.single.id, post.id);
    expect(loaded.notice, 'No relay reachable.');
  });
}
