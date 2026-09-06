import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/live_frame_probe.dart';
import '../../integration_test/support/live_video_frame_policy.dart';

void main() {
  configureLiveVideoFrames(LiveTestWidgetsFlutterBinding());
  testWidgets(
    'live video measurement renders animations between observer polls',
    (tester) async {
      var builds = 0;
      await tester.pumpWidget(LiveFrameProbe(onBuild: () => builds++));
      final initial = builds;
      await Future<void>.delayed(const Duration(milliseconds: 300));
      expect(
        builds - initial,
        greaterThan(1),
        reason: 'a 50ms polling loop must not gate real display rendering',
      );
      await tester.pumpWidget(const SizedBox.shrink());
    },
  );
}
