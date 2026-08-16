import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/block_during_fetch_social_graph.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a block completed during refresh removes its held row', () async {
    final blocked = samplePost(id: 'blocked');
    final fresh = samplePost(
      id: 'fresh',
      creator: sampleCreator(id: 'fresh-creator'),
    );
    final reader = _RefreshGatedReader([blocked, fresh]);
    final social = BlockDuringFetchSocialGraph();
    final feed = FilteredVideoFeedRepository(reader, social);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: const []),
        optional: FeedOptionalDependencies(social: social),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    final refresh = cubit.refresh();
    await reader.refreshStarted.future;
    await social.toggleBlock(blocked.creator.id);
    reader.releaseRefresh.complete();
    await refresh;

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['fresh']);
  });
}

final class _RefreshGatedReader implements VideoPostReader {
  _RefreshGatedReader(this.posts);

  final List<VideoPost> posts;
  final refreshStarted = Completer<void>();
  final releaseRefresh = Completer<void>();
  var _loads = 0;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    _loads += 1;
    if (_loads == 2) {
      refreshStarted.complete();
      await releaseRefresh.future;
    }
    return posts;
  }

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async => const [];
}
