import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/sample_data.dart';

void main() {
  test('rejects a focus outside the feed', () {
    expect(
      () => FeedFocus.around(posts: [samplePost()], activeIndex: 1),
      throwsRangeError,
    );
    expect(
      () => FeedFocus.around(posts: [samplePost()], activeIndex: -1),
      throwsRangeError,
    );
    expect(
      () => FeedFocus.around(posts: const [], activeIndex: 0),
      throwsRangeError,
    );
  });
}
