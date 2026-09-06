import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';

import '../../integration_test/support/device_playback_probe.dart';
import '../../integration_test/support/live_video_log.dart';
import '../../integration_test/support/live_video_swipe.dart';

void main() {
  testWidgets(
    'swipe timing brackets release dispatch at microsecond precision',
    (tester) async {
      final log = LiveVideoLog();
      final focus = DevicePlaybackProbe().markFocus(
        PlaybackVideoId.parse('previous'),
      );
      var released = false;
      final clockPhases = <bool>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Listener(
            onPointerUp: (_) => released = true,
            child: PageView(children: const [Text('First'), Text('Second')]),
          ),
        ),
      );
      final swipe = LiveVideoSwipe(
        tester,
        log,
        clock: () {
          clockPhases.add(released);
          return (
            elapsedUs: released ? 1_236_789 : 1_234_567,
            epochUs: released ? 1_788_651_001_236_789 : 1_788_651_001_234_567,
          );
        },
      );

      await swipe.perform(LiveSwipeDirection.forward, focus);

      expect(clockPhases, [false, true]);
      final event = log.records.single;
      expect(event['type'], 'gesture_release');
      expect(event['fromEventId'], 'previous');
      expect(event['fromSequence'], focus.sequence);
      expect(event['direction'], 'forward');
      expect(event['releaseStartedUs'], 1_234_567);
      expect(event['releaseDispatchedUs'], 1_236_789);
      expect(event['releaseWallUs'], 1_788_651_001_234_567);
      expect(released, isTrue);
    },
  );
}
