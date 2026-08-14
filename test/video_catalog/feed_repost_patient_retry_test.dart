import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_reposts.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test(
    'failed patient hydration releases its target for a later retry',
    () async {
      final post = repostablePost();
      final repository = _FailThenObserve();
      final reposts = FeedReposts(repository);

      expect(await reposts.settle([post]), isEmpty);
      final retried = await reposts.settle([post]);

      expect(repository.calls, 2);
      expect(
        retried.single.repostContext.observation,
        VideoRepostObservation.observed,
      );
    },
  );
}

final class _FailThenObserve implements VideoRepostRepository {
  var calls = 0;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async {
    calls += 1;
    if (calls == 1) throw const AppFailure('offline');
    return [
      posts.single.withRepost(
        true,
        observation: VideoRepostObservation.observed,
      ),
    ];
  }

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
