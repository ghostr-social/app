import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test('absorbs engine failures instead of breaking the feed', () async {
    final updater = RecordingRustFocusUpdater()
      ..failure = StateError('engine offline');
    final port = FfiFeedFocusPort(updateFocus: updater.call);

    port.focusChanged(FeedFocus.around(posts: [samplePost()], activeIndex: 0));
    await pumpEventQueue();

    expect(updater.updates, isEmpty);
  });
}
