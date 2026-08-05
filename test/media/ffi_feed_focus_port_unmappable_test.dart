import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test('drops undeliverable media and keeps the viewer aligned', () async {
    final updater = RecordingRustFocusUpdater();
    final port = FfiFeedFocusPort(updateFocus: updater.call);
    final local = samplePost(id: 'local')
        .withMedia(VideoMediaSource.local('/tmp/clip.mp4'));
    final posts = [local, samplePost(id: 'current'), samplePost(id: 'next')];

    port.focusChanged(FeedFocus.around(posts: posts, activeIndex: 1));
    await pumpEventQueue();

    final update = updater.updates.single;
    expect(update.items.map((item) => item.urls.single), [
      'https://example.com/video/current.mp4',
      'https://example.com/video/next.mp4',
    ]);
    expect(update.currentIndex, 0);
  });

  test('clears Rust focus when no window item is deliverable', () async {
    final updater = RecordingRustFocusUpdater();
    final port = FfiFeedFocusPort(updateFocus: updater.call);
    final local = samplePost(id: 'local')
        .withMedia(VideoMediaSource.local('/tmp/clip.mp4'));

    port.focusChanged(FeedFocus.around(posts: [local], activeIndex: 0));
    await pumpEventQueue();

    final update = updater.updates.single;
    expect(update.items, isEmpty);
    expect(update.currentIndex, 0);
  });
}
