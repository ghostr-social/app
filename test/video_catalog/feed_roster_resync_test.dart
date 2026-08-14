import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('a resync keeps viewer order and appends unseen posts at the tail', () {
    final roster = FeedRoster([
      samplePost(id: 'a'),
      samplePost(id: 'b'),
      samplePost(id: 'c'),
    ], activeIndex: 2);

    final resynced = roster.resynced([
      samplePost(id: 'unseen'),
      samplePost(id: 'c', caption: 'fresh c'),
      samplePost(id: 'a'),
    ]);

    expect(resynced.posts.map((post) => post.id.value), ['a', 'c', 'unseen']);
    expect(resynced.active.caption, 'fresh c');
    expect(resynced.activeIndex, 1);
  });
}
