import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('ordinary navigation retains exactly three previous videos', () {
    final roster = FeedRoster(
      List.generate(6, (index) => samplePost(id: 'post-$index')),
    );

    final moved = roster.movedTo(4, history: FeedNavigationHistory.ordinary);

    expect(moved.posts.map((post) => post.id.value), [
      'post-1',
      'post-2',
      'post-3',
      'post-4',
      'post-5',
    ]);
    expect(moved.active.id.value, 'post-4');
    expect(moved.activeIndex, 3);
  });
}
