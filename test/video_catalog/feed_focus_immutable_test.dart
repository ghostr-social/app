import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/sample_data.dart';

void main() {
  test('does not expose a mutable focus window', () {
    final focus = FeedFocus.around(posts: [samplePost()], activeIndex: 0);

    expect(() => focus.window.clear(), throwsUnsupportedError);
  });
}
