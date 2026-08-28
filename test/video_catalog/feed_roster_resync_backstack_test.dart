import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';

import '../support/sample_data.dart';

void main() {
  test('ordinary resync keeps only the bounded watched backstack', () {
    final posts = List.generate(6, (index) => samplePost(id: 'post-$index'));
    final roster = FeedRoster(posts, activeIndex: 4);
    final retained = posts
        .skip(1)
        .take(4)
        .map(VideoInteractionTarget.fromPost)
        .toSet();

    final refreshed = roster.resynced(
      posts,
      eligible: [posts.last],
      retainWatched: false,
      retainedHeldTargets: retained,
    );

    expect(refreshed.posts.map((post) => post.id.value), [
      'post-1',
      'post-2',
      'post-3',
      'post-4',
      'post-5',
    ]);
    expect(refreshed.active.id.value, 'post-4');
  });
}
