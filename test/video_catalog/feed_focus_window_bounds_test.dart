import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/sample_data.dart';

void main() {
  test('the delivery focus is bounded around the viewer', () {
    final posts = [
      for (var index = 0; index < 50; index += 1) samplePost(id: 'post-$index'),
    ];

    final middle = FeedFocus.around(posts: posts, activeIndex: 25);
    expect(middle.window, posts.sublist(22, 50));
    expect(middle.currentIndex, 3);
    expect(middle.current, posts[25]);

    final start = FeedFocus.around(posts: posts, activeIndex: 0);
    expect(start.window, posts.sublist(0, 25));
    expect(start.currentIndex, 0);
    expect(start.current, posts[0]);

    final end = FeedFocus.around(posts: posts, activeIndex: 49);
    expect(end.window, posts.sublist(46));
    expect(end.currentIndex, 3);
    expect(end.current, posts[49]);
  });
}
