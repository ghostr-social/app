import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_policy.dart';

import '../support/sample_data.dart';

void main() {
  test('keeps only followed and unblocked posts in the following feed', () {
    final followed = sampleCreator(id: 'followed');
    final blocked = sampleCreator(id: 'blocked');

    final posts = const VideoFeedPolicy().select(
      kind: FeedKind.following,
      posts: [samplePost(creator: followed), samplePost(creator: blocked)],
      followed: {followed.id, blocked.id},
      blocked: {blocked.id},
    );

    expect(posts.map((post) => post.creator.id), [followed.id]);
  });
}
