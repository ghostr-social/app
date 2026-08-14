import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_arbiter.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'releasing a lease retains native focus and permanently ignores writes',
    () {
      final sink = FakeFeedFocusPort();
      final lease = FeedFocusArbiter(sink).openLease()..activate();
      lease.focusChanged(_focus('before'));

      lease.release();
      lease.focusChanged(_focus('after'));
      lease.activate();

      expect(sink.focuses.map((focus) => focus.current.id.value), ['before']);
    },
  );
}

FeedFocus _focus(String id) {
  return FeedFocus.around(posts: [samplePost(id: id)], activeIndex: 0);
}
