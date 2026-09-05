import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_arbiter.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'hiding the last feed clears Rust focus with a newer generation',
    () async {
      final updater = RecordingRustFocusUpdater();
      final arbiter = FeedFocusArbiter(
        FfiFeedFocusPort(updateFocus: updater.call),
      );
      final lease = arbiter.openLease()..activate();
      lease.focusChanged(
        FeedFocus.around(posts: [samplePost()], activeIndex: 0),
      );
      await pumpEventQueue();
      final currentGeneration = updater.updates.single.generation;

      lease.deactivate();
      await pumpEventQueue();

      expect(updater.updates.last.items, isEmpty);
      expect(updater.updates.last.watchMs, BigInt.zero);
      expect(updater.updates.last.generation, greaterThan(currentGeneration));
    },
  );
}
