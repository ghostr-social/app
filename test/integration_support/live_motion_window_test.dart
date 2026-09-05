import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/live_motion_window.dart';

void main() {
  test('frozen playback cannot pass because it rendered one early frame', () {
    final motion = LiveMotionWindow();
    motion.record(const Duration(), const Duration(milliseconds: 500));
    motion.record(
      const Duration(seconds: 10),
      const Duration(milliseconds: 500),
    );
    expect(motion.longestFreeze, const Duration(seconds: 10));
    expect(motion.advances, 0);
  });

  test('loop boundaries count as motion without hiding a later freeze', () {
    final motion = LiveMotionWindow();
    motion.record(Duration.zero, const Duration(seconds: 3));
    motion.record(const Duration(seconds: 1), Duration.zero);
    motion.record(const Duration(seconds: 2), const Duration(seconds: 1));
    motion.record(const Duration(seconds: 5), const Duration(seconds: 1));
    expect(motion.advances, 2);
    expect(motion.longestFreeze, const Duration(seconds: 3));
  });
}
