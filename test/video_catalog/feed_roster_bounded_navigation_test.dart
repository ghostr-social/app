import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('ordinary navigation retains exactly five previous videos', () {
    final roster = FeedRoster(
      List.generate(8, (index) => samplePost(id: 'post-$index')),
    );

    final moved = roster.movedTo(6, history: FeedNavigationHistory.ordinary);

    expect(moved.posts.map((post) => post.id.value), [
      'post-1',
      'post-2',
      'post-3',
      'post-4',
      'post-5',
      'post-6',
      'post-7',
    ]);
    expect(moved.active.id.value, 'post-6');
    expect(moved.activeIndex, 5);
  });
}
