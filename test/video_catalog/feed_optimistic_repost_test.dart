import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';

void main() {
  test('flips repost state before the relay mutation completes', () async {
    final post = repostablePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
    final gate = Completer<void>();
    final reposts = _GatedReposts(gate.future);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: feed,
        viewerId: ProfileId.parse('viewer'),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(reposts: reposts),
        ),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    final toggling = cubit.toggleRepost(post);
    expect((cubit.state as FeedLoaded).posts.first.viewerHasReposted, isTrue);

    gate.complete();
    await toggling;
    expect((cubit.state as FeedLoaded).posts.first.viewerHasReposted, isTrue);
  });
}

final class _GatedReposts implements VideoRepostRepository {
  const _GatedReposts(this.gate);
  final Future<void> gate;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async {
    await gate;
    return post.withRepost(!post.viewerHasReposted);
  }
}
