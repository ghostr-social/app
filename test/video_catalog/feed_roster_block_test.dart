import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('dropping a creator keeps the first survivor below the viewer', () {
    final kept = sampleCreator(id: 'creator-kept');
    final blocked = sampleCreator(id: 'creator-blocked');
    final roster = FeedRoster(
      [
        samplePost(id: 'a', creator: kept),
        samplePost(id: 'b', creator: blocked),
        samplePost(id: 'c', creator: blocked),
        samplePost(id: 'd', creator: kept),
      ],
      activeIndex: 1,
    );

    final remaining = roster.withoutCreator(blocked.id);

    expect(remaining.posts.map((post) => post.id.value), ['a', 'd']);
    expect(remaining.active.id.value, 'd');
  });

  test('dropping the creator of every post below falls back to the end', () {
    final kept = sampleCreator(id: 'creator-kept');
    final blocked = sampleCreator(id: 'creator-blocked');
    final roster = FeedRoster(
      [
        samplePost(id: 'a', creator: kept),
        samplePost(id: 'b', creator: blocked),
      ],
      activeIndex: 1,
    );

    final remaining = roster.withoutCreator(blocked.id);

    expect(remaining.active.id.value, 'a');
  });
}
