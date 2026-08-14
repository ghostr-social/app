import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_reposts.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/repost_samples.dart';

void main() {
  test('one target starts at most one patient hydration at a time', () async {
    final gate = Completer<List<VideoPost>>();
    final repository = _GatedHydration(gate.future);
    final reposts = FeedReposts(repository);
    final posts = [repostablePost()];

    final first = reposts.settle(posts);
    final second = await reposts.settle(posts);

    expect(repository.patientCalls, 1);
    expect(second, isEmpty);
    gate.complete(posts);
    await first;
  });
}

final class _GatedHydration implements VideoRepostRepository {
  _GatedHydration(this.result);

  final Future<List<VideoPost>> result;
  var patientCalls = 0;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) {
    if (mode == VideoRepostHydration.patient) patientCalls += 1;
    return result;
  }

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
