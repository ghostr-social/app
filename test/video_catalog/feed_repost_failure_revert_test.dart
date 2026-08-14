import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'failed repost restores the row and exposes the failure notice',
    () async {
      final post = repostablePost();
      final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: feed,
          viewerId: ProfileId.parse('viewer'),
          optional: const FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies(reposts: _FailingReposts()),
          ),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();

      await cubit.toggleRepost(post);

      final state = cubit.state as FeedLoaded;
      expect(state.posts.first.viewerHasReposted, isFalse);
      expect(state.notice, 'Relays are unreachable.');
    },
  );
}

final class _FailingReposts implements VideoRepostRepository {
  const _FailingReposts();

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) {
    throw const AppFailure('Relays are unreachable.');
  }
}
