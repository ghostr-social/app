import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('losing the active post clamps the viewer to the last survivor', () {
    final roster = FeedRoster(
      [samplePost(id: 'a'), samplePost(id: 'b'), samplePost(id: 'c')],
      activeIndex: 2,
    );

    final resynced = roster.resynced([samplePost(id: 'a')]);

    expect(resynced.posts.single.id.value, 'a');
    expect(resynced.activeIndex, 0);
    expect(resynced.ahead, 0);
  });

  test('losing the active post holds the position it still fits in', () {
    final roster = FeedRoster(
      [samplePost(id: 'a'), samplePost(id: 'b'), samplePost(id: 'c')],
      activeIndex: 1,
    );

    final resynced =
        roster.resynced([samplePost(id: 'a'), samplePost(id: 'c')]);

    expect(resynced.active.id.value, 'c');
    expect(resynced.activeIndex, 1);
  });

  test('a resync that loses everything leaves an empty roster', () {
    final roster = FeedRoster([samplePost(id: 'a')]);

    expect(roster.resynced(const []).isEmpty, isTrue);
  });
}
