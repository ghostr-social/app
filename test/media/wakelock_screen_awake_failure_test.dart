import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/wakelock_screen_awake.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

import '../support/fake_wakelock_platform.dart';

void main() {
  test('wakelock failures never propagate into playback', () async {
    wakelockPlusPlatformInstance = FakeWakelockPlatform(
      failure: PlatformException(code: 'wakelock-unavailable'),
    );
    const screen = WakelockScreenAwake();

    await expectLater(screen.enable(), completes);
    await expectLater(screen.disable(), completes);
  });
}
