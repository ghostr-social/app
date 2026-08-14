import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_policy.dart';

import '../support/repost_samples.dart';

void main() {
  test('Following admits by reposter and blocks either responsible actor', () {
    final post = repostedPost();
    const policy = VideoFeedPolicy();
    final reposter = post.repost!.reposter.id;

    final visible = policy.select(
      kind: FeedKind.following,
      posts: [post],
      followed: {reposter},
      blocked: const {},
    );
    final blocked = policy.select(
      kind: FeedKind.following,
      posts: [post],
      followed: {reposter},
      blocked: {reposter},
    );

    expect(visible, [post]);
    expect(blocked, isEmpty);
  });
}
