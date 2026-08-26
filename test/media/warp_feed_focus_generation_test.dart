import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import '../../integration_test/support/device_playback_probe.dart';
import '../../integration_test/support/warp_feed_focus_probe.dart';
import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test('focus occurrence retains its exact Rust generation', () async {
    final updater = RecordingRustFocusUpdater();
    final rust = FfiFeedFocusPort(updateFocus: updater.call);
    final probe = WarpFeedFocusProbe(
      rust,
      DevicePlaybackProbe(),
      () => rust.lastScheduledGeneration,
    );

    probe.focusChanged(
      FeedFocus.around(posts: [samplePost(id: 'first')], activeIndex: 0),
    );
    await pumpEventQueue();

    final occurrence = probe.occurrences.single;
    expect(probe.generationFor(occurrence), updater.updates.single.generation);
  });
}
