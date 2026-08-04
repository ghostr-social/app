import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/sample_data.dart';

void main() {
  test('the focus window clips to the feed bounds around the viewer', () {
    final posts = [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ];

    final middle = FeedFocus.around(posts: posts, activeIndex: 5);
    expect(middle.window, posts.sublist(3, 12));
    expect(middle.currentIndex, 2);
    expect(middle.current, posts[5]);

    final start = FeedFocus.around(posts: posts, activeIndex: 0);
    expect(start.window, posts.sublist(0, 7));
    expect(start.currentIndex, 0);
    expect(start.current, posts[0]);

    final end = FeedFocus.around(posts: posts, activeIndex: 11);
    expect(end.window, posts.sublist(9, 12));
    expect(end.currentIndex, 2);
    expect(end.current, posts[11]);
  });
}
