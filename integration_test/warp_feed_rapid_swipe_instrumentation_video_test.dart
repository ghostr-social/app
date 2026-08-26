import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:integration_test/integration_test.dart';
import 'support/device_playback_probe.dart';
import 'support/progressive_device_origin.dart';
import 'support/warp_feed_playback_journey.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('real Android feed preserves playback through a ready burst', (
    tester,
  ) async {
    final journey = await WarpFeedPlaybackJourney.start(
      eventCount: 7,
      validator: ProgressiveOriginValidator.stableStrong,
      dataUsage: DataUsageLevel.aggressive,
      responseChunkDelay: const Duration(milliseconds: 100),
    );
    addTearDown(journey.close);
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    await journey.waitForPostCount(tester, 4);
    final futurePaths = journey.futureRemotePaths(3);
    final startup = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, startup);
    await journey.waitForPlaying(tester, startup);
    final playback = journey.telemetry.probe;
    final planningFocus = await journey.waitForPublishedFocus(
      tester,
      0,
      afterSequence: startup.sequence,
      cause: FeedFocusCause.rosterChange,
    );
    final generation = journey.focus.generationFor(planningFocus)!;
    final planningDeliveryId = playback.activationFor(startup)!.deliveryId;
    final overlap = await journey.waitForParallelBytes(tester, futurePaths);
    final afterOverlap = journey.preparation.latest.sequence;
    final ready = await journey.waitForReadyWindow(
      tester,
      generation,
      currentDeliveryId: planningDeliveryId,
      minimumDepth: 3,
      afterSequence: afterOverlap,
    );
    journey.reportPlan(ready.plan);
    journey.reportParallelPreparation(ready, overlap);
    final focuses = await journey.swipeForward(
      tester,
      count: ready.snapshot.contiguousReadyDepth,
      afterSequence: journey.focusCursor,
    );
    final burstFinal = focuses.last;
    await journey.waitForFirstFrame(tester, burstFinal);
    await journey.waitForPlaying(tester, burstFinal);
    for (final focus in focuses) {
      expect(journey.isReadyIn(ready.snapshot, focus), isTrue);
      expect(playback.firstFrameLatency(focus), isNotNull);
      expect(playback.playingLatency(focus), isNotNull);
      expect(playback.hasPhaseFor(focus, PlaybackPhase.failed), isFalse);
      expect(
        playback.hasPhaseFor(focus, PlaybackPhase.networkStalled),
        isFalse,
      );
    }
    final replenished = await journey.waitForReplenishment(
      tester,
      burstFinal,
      afterRevision: ready.plan.revision,
    );
    final nextIndex = focuses.length + 1;
    await journey.swipeUp(tester);
    final next = await journey.waitForPublishedFocus(
      tester,
      nextIndex,
      afterSequence: burstFinal.sequence,
    );
    await journey.waitForFirstFrame(tester, next);
    await journey.waitForPlaying(tester, next);
    await journey.telemetry.settled;
    expect(journey.isReadyIn(ready.snapshot, next), isFalse);
    expect(journey.isReadyIn(replenished.snapshot, next), isTrue);
    final position = playback.latestPositionFor(next)!;
    await journey.pumpFor(tester, const Duration(seconds: 1));
    journey.reportBurst(ready, replenished, focuses, next);
    expect(playback.latestPositionFor(next), greaterThan(position));
    expect(journey.hadPlaybackError, isFalse);
    expect(journey.focus.hadTransportRescue, isFalse);
  });
}
