import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import '../../integration_test/support/device_playback_probe.dart';
import '../../integration_test/support/warp_feed_focus_probe.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  test('repeated feed destinations remain distinct ordered occurrences', () {
    final delegate = FakeFeedFocusPort();
    final probe = WarpFeedFocusProbe(delegate, DevicePlaybackProbe());

    probe.focusChanged(_focus('first'));
    probe.focusChanged(_focus('first', cause: FeedFocusCause.rosterChange));
    probe.focusChanged(_focus('second'));
    probe.focusChanged(_focus('first'));

    expect(probe.occurrences.map((item) => item.videoId.value), [
      'first',
      'first',
      'second',
      'first',
    ]);
    expect(
      probe
          .occurrencesFor('first', cause: FeedFocusCause.userNavigation)
          .length,
      2,
    );
    expect(
      probe.occurrenceAfter(
        'first',
        probe.occurrences.first.sequence,
        cause: FeedFocusCause.userNavigation,
      ),
      same(probe.occurrences.last),
    );
    expect(probe.publishedFor('first'), same(probe.occurrences.last));
    expect(delegate.focuses.map((item) => item.current.id.value), [
      'first',
      'first',
      'second',
      'first',
    ]);
  });
}

FeedFocus _focus(
  String id, {
  FeedFocusCause cause = FeedFocusCause.userNavigation,
}) {
  return FeedFocus.around(
    posts: [samplePost(id: id)],
    activeIndex: 0,
    cause: cause,
  );
}
