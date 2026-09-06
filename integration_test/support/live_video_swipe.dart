import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'live_video_log.dart';

enum LiveSwipeDirection {
  forward(-1),
  backward(1);

  const LiveSwipeDirection(this.offset);
  final double offset;
}

typedef LiveSwipeInstant = ({int elapsedUs, int epochUs});
typedef LiveSwipeClock = LiveSwipeInstant Function();

/// Brackets synthetic pointer release; display presentation is measured separately.
final class LiveVideoSwipe {
  LiveVideoSwipe(this.tester, this.log, {LiveSwipeClock? clock})
    : _clock =
          clock ??
          (() => (
            elapsedUs: log.watch.elapsedMicroseconds,
            epochUs: DateTime.now().microsecondsSinceEpoch,
          ));

  final WidgetTester tester;
  final LiveVideoLog log;
  final LiveSwipeClock _clock;

  Future<void> perform(
    LiveSwipeDirection direction,
    PlaybackFocus? from,
  ) async {
    final page = find.byType(PageView).first;
    final distance = tester.getSize(page).height * 0.23 * direction.offset;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, distance));
    await tester.pump(deviceRapidSwipeGestureTarget);
    final start = _clock();
    await gesture.up();
    final end = _clock();
    _recordRelease(direction, from, (start: start, end: end));
  }

  void _recordRelease(
    LiveSwipeDirection direction,
    PlaybackFocus? from,
    ({LiveSwipeInstant start, LiveSwipeInstant end}) timing,
  ) {
    log.add('gesture_release', {
      'fromEventId': from?.videoId.value,
      'fromSequence': from?.sequence,
      'direction': direction.name,
      'releaseStartedUs': timing.start.elapsedUs,
      'releaseDispatchedUs': timing.end.elapsedUs,
      'releaseWallUs': timing.start.epochUs,
    });
  }
}
