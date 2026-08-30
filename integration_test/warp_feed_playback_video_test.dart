import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_qoe_targets.dart';
import 'support/device_playback_probe.dart';
import 'support/progressive_device_origin.dart';
import 'support/warp_feed_playback_journey.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('signed Ghostr feed plays initial and final rapid-swipe focus', (
    tester,
  ) async {
    final journey = await WarpFeedPlaybackJourney.start();
    addTearDown(journey.close);
    final startup = await _openSignedFeed(tester, journey);
    expect(journey.feedRepository, isA<AccountScopedVideoFeedRepository>());
    _expectSignedFeed(journey, startup);
    final finalFocus = await _rapidSwipes(tester, journey);
    await _expectInitialAndFinalPlayback(tester, journey, startup, finalFocus);
  });
}

Future<PlaybackFocus> _openSignedFeed(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  final parallelPrefixes = journey.resources.origin.rendezvousFirstChunks({
    '/current.mp4',
    '/next.mp4',
  });
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstChunkRendezvous(tester, parallelPrefixes);
  await journey.waitForFirstFrame(tester, startup);
  await journey.waitForPlaying(tester, startup);
  await journey.waitForPreparation(tester);
  await journey.waitForParallelRangedVideos(tester);
  return startup;
}

void _expectSignedFeed(WarpFeedPlaybackJourney journey, PlaybackFocus startup) {
  expect(journey.relay.videoSubscriptions, greaterThan(0));
  final state = journey.cubit.state as FeedLoaded;
  expect(
    state.posts.map((post) => post.id.value),
    journey.events.map((event) => event.id),
  );
  expect(
    state.posts.every((post) => post.nostrReference?.signedEvent != null),
    isTrue,
  );
  journey.reportStartup(startup);
}

Future<PlaybackFocus> _rapidSwipes(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await journey.swipeUp(tester);
  await journey.swipeUp(tester);
  await journey.waitForCaption(tester, 2);
  final finalFocus = await journey.waitForPublishedFocus(tester, 2);
  await journey.waitForFirstFrame(tester, finalFocus);
  await journey.waitForPlaying(tester, finalFocus);
  return finalFocus;
}

Future<void> _expectInitialAndFinalPlayback(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus startup,
  PlaybackFocus finalFocus,
) async {
  final initial = journey.telemetry.probe.latestPosition(finalFocus.videoId);
  await journey.pumpFor(tester, const Duration(seconds: 1));
  journey.reportFinal(finalFocus);
  expect(
    journey.telemetry.probe.firstFrameLatency(startup),
    lessThan(deviceStartupTarget),
  );
  expect(
    journey.telemetry.probe.firstFrameLatency(finalFocus),
    lessThan(deviceFocusSwitchTarget),
  );
  expect(
    journey.telemetry.probe.latestPosition(finalFocus.videoId),
    greaterThan(initial),
  );
  expect(find.text('Video unavailable'), findsNothing);
}
