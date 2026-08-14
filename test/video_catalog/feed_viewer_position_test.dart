import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a changed roster republishes focus without another watch', () async {
    final focus = FakeFeedFocusPort();
    final history = FakeWatchHistoryRepository();
    final viewer = FeedViewer(
      focus: focus,
      watchTracker: WatchHistoryTracker(
        history: history,
        settings: FakeAppSettingsRepository(AppSettings.defaults()),
        failureReporter: RecordingFailureReporter(),
      ),
    );
    final posts = [samplePost(id: 'one'), samplePost(id: 'two')];

    viewer.landedOn(posts, 1);
    await pumpEventQueue();

    expect(focus.focuses.single.current.id.value, 'two');
    expect(history.entries.map((entry) => entry.videoId), ['e:two']);

    viewer.rosterChanged(posts, 0);
    await pumpEventQueue();

    expect(focus.focuses, hasLength(2));
    expect(focus.focuses.last.current.id.value, 'one');
    expect(history.entries.map((entry) => entry.videoId), ['e:two']);
  });
}
