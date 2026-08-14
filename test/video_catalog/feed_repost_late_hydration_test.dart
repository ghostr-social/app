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
  test('loaded feed corrects repost state after patient hydration', () async {
    final post = repostablePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
    final late = Completer<List<VideoPost>>();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: feed,
        viewerId: ProfileId.parse('viewer'),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(reposts: _LateReposts(late)),
        ),
      ),
    );
    addTearDown(cubit.close);

    await cubit.load();
    expect(_post(cubit).viewerHasReposted, isFalse);
    final corrected = post.withRepost(
      true,
      observation: VideoRepostObservation.observed,
    );
    late.complete([corrected]);
    await cubit.stream.firstWhere((state) => _reposted(state));

    expect(_post(cubit).viewerHasReposted, isTrue);
  });
}

VideoPost _post(FeedCubit cubit) => (cubit.state as FeedLoaded).posts.single;

bool _reposted(FeedState state) =>
    state is FeedLoaded && state.posts.single.viewerHasReposted;

final class _LateReposts implements VideoRepostRepository {
  const _LateReposts(this.late);
  final Completer<List<VideoPost>> late;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) =>
      mode == VideoRepostHydration.patient ? late.future : Future.value(posts);

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
