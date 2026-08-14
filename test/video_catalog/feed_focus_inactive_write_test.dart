import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_arbiter.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  test('a late write from an inactive lease cannot replace active focus', () {
    final sink = FakeFeedFocusPort();
    final arbiter = FeedFocusArbiter(sink);
    final home = arbiter.openLease()..activate();
    final routed = arbiter.openLease()..activate();
    routed.focusChanged(_focus('routed'));

    home.focusChanged(_focus('late-home'));

    expect(sink.focuses, hasLength(1));
    expect(sink.focuses.single.current.id.value, 'routed');
  });
}

FeedFocus _focus(String id) {
  return FeedFocus.around(posts: [samplePost(id: id)], activeIndex: 0);
}
