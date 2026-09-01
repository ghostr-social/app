import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';

Future<void> runWarpIgnoredRangeScenario(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    options: const WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 3),
      dataUsage: DataUsageLevel.aggressive,
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
        rangeSemanticsById: {
          'current': ProgressiveOriginRangeSemantics.ignored,
        },
      ),
    ),
  );
  addTearDown(journey.close);

  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  final focus = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, focus);
  await journey.waitForPlaying(tester, focus);
  await journey.waitForNativeStoreCoverage(tester, ['current']);
  await journey.waitForOriginQuiescence(tester, ['current']);

  _expectBoundedRecovery(journey.resources.origin);
  journey.expectSinglePlayerAttempt(focus);
  expect(find.text('Video unavailable'), findsNothing);
}

void _expectBoundedRecovery(ProgressiveDeviceOrigin origin) {
  final bodies = origin
      .requestsFor('current')
      .where((request) => request.method == 'GET')
      .toList();
  final evidence = bodies.map(_requestSummary).join(', ');
  expect(bodies.length, inInclusiveRange(1, 2), reason: evidence);
  _expectInitialRange(bodies.first, evidence);
  _expectTerminalBodies(bodies, evidence);
  _expectFinalBody(bodies, origin.objectLength, evidence);
  _expectCoverage(origin, evidence);
  debugPrint('WARP_IGNORED_RANGE bodies=$evidence');
}

void _expectInitialRange(ProgressiveOriginRequest body, String evidence) {
  expect(body.range?.start, 0, reason: evidence);
}

void _expectTerminalBodies(
  List<ProgressiveOriginRequest> bodies,
  String evidence,
) {
  expect(
    bodies.every(
      (body) => body.outcome != ProgressiveOriginRequestOutcome.serving,
    ),
    isTrue,
    reason: evidence,
  );
}

void _expectFinalBody(
  List<ProgressiveOriginRequest> bodies,
  int objectLength,
  String evidence,
) {
  final finalBody = bodies.last;
  if (bodies.length == 2) expect(finalBody.range, isNull, reason: evidence);
  expect(finalBody.outcome, ProgressiveOriginRequestOutcome.completed);
  expect(finalBody.servedBytes, objectLength, reason: evidence);
}

void _expectCoverage(ProgressiveDeviceOrigin origin, String evidence) {
  final coverage = origin.coverageFor('current');
  expect(coverage.isComplete, isTrue, reason: evidence);
  expect(coverage.isWithinObject, isTrue, reason: evidence);
  expect(coverage.uniqueBytes, origin.objectLength, reason: evidence);
  expect(
    coverage.networkBytes,
    lessThanOrEqualTo(origin.objectLength * 2),
    reason: evidence,
  );
}

String _requestSummary(ProgressiveOriginRequest request) {
  final range = request.range;
  return '${range?.start}-${range?.end} '
      '${request.servedBytes} ${request.outcome.name}';
}
