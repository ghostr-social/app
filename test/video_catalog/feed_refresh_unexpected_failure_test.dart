import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('an unexpected refresh failure preserves the loaded feed', () async {
    final repository = _UnexpectedRefreshRepository();
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.refresh();

    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(1));
    expect(state.notice, 'Could not load the Nostr video feed.');
  });
}

class _UnexpectedRefreshRepository extends FakeVideoCatalogRepository {
  _UnexpectedRefreshRepository() : super(forYouFeed: [samplePost()]);

  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) async {
    if (loads++ > 0) throw StateError('disk failure');
    return super.loadFeed(kind);
  }
}
