import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe notice for an unexpected older-page error', () async {
    final repository = _BrokenOlderFeedRepository([
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.loadMore();

    expect(
      (cubit.state as FeedLoaded).notice,
      'Could not load the Nostr video feed.',
    );
  });
}

class _BrokenOlderFeedRepository extends FakeVideoCatalogRepository {
  _BrokenOlderFeedRepository(List<VideoPost> posts) : super(forYouFeed: posts);

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    throw StateError('page decoder crashed');
  }
}
