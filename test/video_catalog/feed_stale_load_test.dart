import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an older feed completion cannot replace the latest selection',
      () async {
    final repository = _PendingFeedRepository();
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));

    final older = cubit.load(FeedKind.forYou);
    final latest = cubit.load(FeedKind.following);
    repository.complete(FeedKind.following, [samplePost(id: 'following')]);
    await latest;
    repository.complete(FeedKind.forYou, [samplePost(id: 'stale')]);
    await older;

    final state = cubit.state as FeedLoaded;
    expect(state.kind, FeedKind.following);
    expect(state.posts.single.id.value, 'following');
    await cubit.close();
  });
}

class _PendingFeedRepository extends FakeVideoCatalogRepository {
  _PendingFeedRepository() : super(forYouFeed: []);

  final pending = {
    for (final kind in FeedKind.values) kind: Completer<List<VideoPost>>(),
  };

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
          {bool excludeWatched = false}) =>
      pending[kind]!.future;

  void complete(FeedKind kind, List<VideoPost> posts) {
    pending[kind]!.complete(posts);
  }
}
