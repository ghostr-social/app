import 'package:flutter_test/flutter_test.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';
import '../support/scripted_playback_preparation_updates.dart';

void main() {
  testWidgets('a completed preparation stream falls back to direct playback', (
    tester,
  ) async {
    await _expectDirectPlayback(tester, () => const Stream.empty());
  });

  testWidgets('an errored preparation stream falls back to direct playback', (
    tester,
  ) async {
    await _expectDirectPlayback(
      tester,
      () => Stream.error(StateError('native watcher failed')),
    );
  });

  testWidgets('a synchronous watch failure falls back to direct playback', (
    tester,
  ) async {
    await _expectDirectPlayback(
      tester,
      () => throw StateError('native watcher unavailable'),
    );
  });
}

Future<void> _expectDirectPlayback(
  WidgetTester tester,
  PreparationWatch watch,
) async {
  final playback = FakeVideoPlaybackPort();
  await tester.pumpWidget(
    feedScreenHarness(
      FakeVideoCatalogRepository(forYouFeed: [samplePost(id: 'current')]),
      options: FeedScreenHarnessOptions(
        playbackPort: playback,
        preparationUpdates: ScriptedPlaybackPreparationUpdates(watch),
      ),
    ),
  );
  await tester.pumpAndSettle();

  expect(playback.requests, hasLength(1));
  expect(
    playback.requests.single.media.remoteUrl,
    'https://example.com/video/current.mp4',
  );
}
