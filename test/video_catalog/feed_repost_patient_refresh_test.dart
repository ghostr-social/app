import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';

void main() {
  test('manual refresh rechecks a previously settled repost target', () async {
    final post = repostablePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
    final reposts = _RefreshReposts();
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
    await Future<void>.delayed(Duration.zero);

    await cubit.refresh();
    reposts.remote.complete();
    await cubit.stream
        .firstWhere(_reposted)
        .timeout(const Duration(milliseconds: 100));

    expect(reposts.patientCalls, 2);
  });
}

bool _reposted(FeedState state) =>
    state is FeedLoaded && state.posts.single.viewerHasReposted;

final class _RefreshReposts implements VideoRepostRepository {
  final remote = Completer<void>();
  var patientCalls = 0;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async {
    if (mode == VideoRepostHydration.prompt) return posts;
    patientCalls += 1;
    if (patientCalls > 1) await remote.future;
    return [
      posts.single.withRepost(
        patientCalls > 1,
        observation: VideoRepostObservation.observed,
      ),
    ];
  }

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
