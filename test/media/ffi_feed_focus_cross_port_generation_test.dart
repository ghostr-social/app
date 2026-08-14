import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test('separate focus ports share a process-monotonic generation', () async {
    final firstUpdater = RecordingRustFocusUpdater();
    final secondUpdater = RecordingRustFocusUpdater();
    final first = FfiFeedFocusPort(updateFocus: firstUpdater.call);
    final second = FfiFeedFocusPort(updateFocus: secondUpdater.call);

    first.focusChanged(FeedFocus.around(posts: [samplePost()], activeIndex: 0));
    await pumpEventQueue();
    second.focusChanged(
      FeedFocus.around(posts: [samplePost()], activeIndex: 0),
    );
    await pumpEventQueue();

    expect(
      secondUpdater.updates.single.generation,
      greaterThan(firstUpdater.updates.single.generation),
    );
  });
}
