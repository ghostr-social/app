import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_qoe_targets.dart';
import 'support/device_playback_probe.dart';
import 'support/warp_feed_playback_journey.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('signed Ghostr feed prepares and plays across rapid swipes', (
    tester,
  ) async {
    final journey = await WarpFeedPlaybackJourney.start();
    addTearDown(journey.close);
    final startup = await _openSignedFeed(tester, journey);
    _expectSignedFeed(journey, startup);
    final finalFocus = await _rapidSwipes(tester, journey);
    await _expectContinuousFinalPlayback(tester, journey, finalFocus);
  });
}

Future<PlaybackFocus> _openSignedFeed(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForPlaying(tester, startup);
  await journey.waitForPreparation(tester);
  await journey.waitForParallelRangedVideos(tester);
  return startup;
}

void _expectSignedFeed(WarpFeedPlaybackJourney journey, PlaybackFocus startup) {
  expect(journey.relay.videoSubscriptions, greaterThan(0));
  expect(
    journey.telemetry.probe.playingLatency(startup),
    lessThan(deviceStartupTarget),
  );
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
  journey.markFocus(1);
  await journey.swipeUp(tester);
  journey.markFocus(0);
  await journey.swipeDown(tester);
  journey.markFocus(1);
  await journey.swipeUp(tester);
  final finalFocus = journey.markFocus(2);
  await journey.swipeUp(tester);
  await journey.waitForCaption(tester, 2);
  await journey.waitForPlaying(tester, finalFocus);
  return finalFocus;
}

Future<void> _expectContinuousFinalPlayback(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus finalFocus,
) async {
  final initial = journey.telemetry.probe.latestPosition(finalFocus.videoId);
  await journey.pumpFor(tester, const Duration(seconds: 1));
  expect(
    journey.telemetry.probe.playingLatency(finalFocus),
    lessThan(deviceFocusSwitchTarget),
  );
  expect(
    journey.telemetry.probe.rebufferRatio,
    lessThanOrEqualTo(deviceRebufferTarget),
  );
  expect(
    journey.telemetry.probe.latestPosition(finalFocus.videoId),
    greaterThan(initial),
  );
  expect(journey.preparation.maximumReadyDepth, greaterThanOrEqualTo(1));
  expect(find.text('Video unavailable'), findsNothing);
  journey.reportFinal(finalFocus);
}
