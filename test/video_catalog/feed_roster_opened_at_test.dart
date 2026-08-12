import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

import '../support/sample_data.dart';

void main() {
  test('a roster opens standing on the chosen video', () {
    final posts = [
      samplePost(id: 'clip-1'),
      samplePost(id: 'clip-2'),
      samplePost(id: 'clip-3'),
    ];
    final roster = FeedRoster(posts);

    expect(roster.openedAt(posts[2].id).activeIndex, 2);
    expect(roster.openedAt(null).activeIndex, 0);
    expect(roster.openedAt(VideoPostId.parse('missing')).activeIndex, 0);
  });
}
