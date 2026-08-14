import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_arbiter.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  test('reactivating a known lease replaces focus without an empty window', () {
    final sink = FakeFeedFocusPort();
    final arbiter = FeedFocusArbiter(sink);
    final home = arbiter.openLease()..activate();
    home.focusChanged(_focus('home'));
    final routed = arbiter.openLease()..activate();
    routed.focusChanged(_focus('routed'));

    home.activate();

    expect(sink.focuses.map((focus) => focus.current.id.value), [
      'home',
      'routed',
      'home',
    ]);
  });
}

FeedFocus _focus(String id) {
  return FeedFocus.around(posts: [samplePost(id: id)], activeIndex: 0);
}
