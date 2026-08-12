import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/wakelock_screen_awake.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

import '../support/fake_wakelock_platform.dart';

void main() {
  test('screen awake requests drive the platform wakelock', () async {
    final platform = FakeWakelockPlatform();
    wakelockPlusPlatformInstance = platform;
    // Deliberately non-const so the constructor line executes at runtime.
    final screen = WakelockScreenAwake();

    await screen.enable();
    expect(platform.toggles, [true]);

    await screen.disable();
    expect(platform.toggles, [true, false]);
  });
}
