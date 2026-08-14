import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';

void main() {
  test('failed completion does not mask later observed relay truth', () async {
    final post = repostablePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: feed,
        viewerId: ProfileId.parse('viewer'),
        optional: const FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(reposts: _AcceptedThenFailed()),
        ),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.toggleRepost(post);
    expect((cubit.state as FeedLoaded).posts.single.viewerHasReposted, isFalse);

    feed.forYouFeed[0] = post.withRepost(
      true,
      observation: VideoRepostObservation.observed,
    );
    await cubit.refresh();

    expect((cubit.state as FeedLoaded).posts.single.viewerHasReposted, isTrue);
  });
}

final class _AcceptedThenFailed implements VideoRepostRepository {
  const _AcceptedThenFailed();

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) {
    throw const AppFailure('The active account changed. Try again.');
  }
}
