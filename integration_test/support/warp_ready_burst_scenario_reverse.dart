part of 'warp_ready_burst_scenario.dart';

Future<void> _consumeBackward(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus from,
) async {
  final ids = _previousOriginIds(journey, 3);
  final before = await journey.waitForOriginQuiescence(tester, ids);
  final reverse = await _reversePlayback(tester, journey, from);
  await _verifyReverseReuse(tester, journey, ids, before, reverse.focuses.last);
}

Future<WarpSwipeBurst> _reversePlayback(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus from,
) async {
  final startedAt = journey.telemetry.probe.elapsed;
  final reverse = await journey.swipeBackward(
    tester,
    count: 3,
    afterSequence: from.sequence,
    cadence: deviceRapidSwipeCadence,
  );
  journey.verifyRapidCadence(startedAt, reverse);
  final finalFocus = reverse.focuses.last;
  await journey.waitForFirstFrame(tester, finalFocus);
  await journey.waitForPlaying(tester, finalFocus);
  journey.verifyReversePlayback(reverse.focuses, reverse.releases);
  return reverse;
}

Future<void> _verifyReverseReuse(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  List<String> ids,
  WarpOriginSnapshot before,
  PlaybackFocus finalFocus,
) async {
  await journey.telemetry.settled;
  final position = journey.telemetry.probe.latestPositionFor(finalFocus)!;
  await journey.pumpFor(tester, const Duration(seconds: 1));
  expect(
    journey.telemetry.probe.latestPositionFor(finalFocus),
    greaterThan(position),
  );
  final after = await journey.waitForOriginQuiescence(tester, ids);
  debugPrint('WARP_REVERSE origin_before=$before origin_after=$after');
  debugPrint('WARP_REVERSE_REQUESTS ${journey.originRequestEvidence(ids)}');
  journey.verifyReplayOriginUse(before, after);
  await journey.waitForReplayStoreCoverage(tester, ids);
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
}

List<String> _previousOriginIds(WarpFeedPlaybackJourney journey, int count) {
  final state = journey.cubit.state as FeedLoaded;
  if (state.activeIndex < count) throw RangeError.value(count);
  return List.generate(count, (offset) {
    final post = state.posts[state.activeIndex - offset - 1];
    final file = Uri.parse(post.media.remoteUrl!).pathSegments.last;
    return file.substring(0, file.length - '.mp4'.length);
  });
}
