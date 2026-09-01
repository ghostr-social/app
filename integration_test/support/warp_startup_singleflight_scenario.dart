import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';

const _nextId = 'next';
const _startupPrefix = (start: 0, end: 65536);

Future<void> runWarpStartupSingleflightScenario(WidgetTester tester) async {
  final journey = await _openStartupFeed(tester);
  await _waitForNextGet(tester, journey);
  final roster = await journey.waitForPublishedFocus(
    tester,
    0,
    cause: FeedFocusCause.rosterChange,
  );
  final generation = journey.focus.generationFor(roster)!;
  await journey.waitForPlan(
    tester,
    (plan) => plan.coversFocusGeneration(generation),
  );
  await journey.waitForOriginQuiescence(tester, [_nextId]);
  await journey.reportSchedulingEvidence();
  _expectStartupSingleflight(journey);
}

Future<WarpFeedPlaybackJourney> _openStartupFeed(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    options: const WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 10),
      dataUsage: DataUsageLevel.aggressive,
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
        pacing: ProgressiveOriginPacing.sharedBandwidth(2500),
      ),
    ),
  );
  addTearDown(journey.close);
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 10);
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, startup);
  await journey.waitForPlaying(tester, startup);
  journey.reportStartup(startup);
  return journey;
}

Future<void> _waitForNextGet(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  final watch = Stopwatch()..start();
  while (!_hasNextGet(journey) && watch.elapsed < const Duration(seconds: 15)) {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  if (_hasNextGet(journey)) return;
  await journey.reportSchedulingEvidence();
  fail('Next reserve request did not start.');
}

bool _hasNextGet(WarpFeedPlaybackJourney journey) => journey.resources.origin
    .requestsFor(_nextId)
    .any((request) => request.method == 'GET');

void _expectStartupSingleflight(WarpFeedPlaybackJourney journey) {
  final origin = journey.resources.origin;
  final gets = origin
      .requestsFor(_nextId)
      .where((item) => item.method == 'GET');
  final prefixes = gets.where(_startsAtZero).toList();
  final evidence = journey.originRequestEvidence([_nextId]);
  expect(gets, isNotEmpty, reason: evidence);
  expect(prefixes, hasLength(1), reason: evidence);
  _expectCompletedPrefix(prefixes.single, evidence);
  _expectExactCoverage(journey, evidence);
}

void _expectCompletedPrefix(ProgressiveOriginRequest prefix, String evidence) {
  expect(prefix.range, _startupPrefix, reason: evidence);
  expect(
    prefix.outcome,
    ProgressiveOriginRequestOutcome.completed,
    reason: evidence,
  );
  expect(prefix.servedBytes, _startupPrefix.end, reason: evidence);
}

void _expectExactCoverage(WarpFeedPlaybackJourney journey, String evidence) {
  final coverage = journey.resources.origin.coverageFor(_nextId);
  expect(coverage.isWithinObject, isTrue, reason: evidence);
  expect(coverage.duplicateBytes, 0, reason: evidence);
  debugPrint(
    'WARP_SINGLEFLIGHT network=${coverage.networkBytes} '
    'unique=${coverage.uniqueBytes} duplicate=${coverage.duplicateBytes} '
    'requests=$evidence chunks=${journey.originChunkEvidence([_nextId])}',
  );
}

bool _startsAtZero(ProgressiveOriginRequest request) =>
    request.range == null || request.range!.start == 0;
