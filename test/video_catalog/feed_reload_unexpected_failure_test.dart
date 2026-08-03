import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a reload hit by an unexpected error keeps the feed with a notice',
      () async {
    final repository = _FlakyReloadRepository([samplePost()]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();
    repository.failNextLoad = true;

    await cubit.reload();

    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(1));
    expect(state.notice, 'Could not load the Nostr video feed.');
  });
}

class _FlakyReloadRepository extends FakeVideoCatalogRepository {
  _FlakyReloadRepository(List<VideoPost> posts) : super(forYouFeed: posts);

  bool failNextLoad = false;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    if (failNextLoad) throw StateError('relay pool crashed');
    return super.loadFeed(kind, excludeWatched: excludeWatched);
  }
}
