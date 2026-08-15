import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'labels a transport-selected focus separately from user navigation',
    () async {
      final updater = RecordingRustFocusUpdater();
      final port = FfiFeedFocusPort(updateFocus: updater.call);

      port.focusChanged(
        FeedFocus.around(
          posts: [samplePost()],
          activeIndex: 0,
          cause: FeedFocusCause.transportRescue,
        ),
      );
      await pumpEventQueue();

      expect(
        updater.updates.single.transition,
        FfiFocusTransition.transportRescue,
      );
    },
  );
}
