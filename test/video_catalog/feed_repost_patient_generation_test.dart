import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_reposts.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test(
    'a patient result from a forgotten feed generation is ignored',
    () async {
      final post = repostablePost();
      final gate = Completer<List<VideoPost>>();
      final reposts = FeedReposts(_GatedHydration(gate.future));

      final settling = reposts.settle([post]);
      reposts.forget();
      gate.complete([
        post.withRepost(true, observation: VideoRepostObservation.observed),
      ]);

      expect(await settling, isEmpty);
    },
  );
}

final class _GatedHydration implements VideoRepostRepository {
  const _GatedHydration(this.result);
  final Future<List<VideoPost>> result;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) => result;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
