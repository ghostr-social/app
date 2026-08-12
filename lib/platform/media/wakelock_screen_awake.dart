import 'dart:developer';

import 'package:ghostr/features/video_inventory/domain/screen_awake_port.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

const _logName = 'ghostr.video.wakelock';

/// Drives the platform wakelock so the screen survives untouched playback.
///
/// Wakelock failures are logged and absorbed; losing the wakelock must never
/// interrupt playback itself.
final class WakelockScreenAwake implements ScreenAwakePort {
  const WakelockScreenAwake();

  @override
  Future<void> enable() => _toggle(enable: true);

  @override
  Future<void> disable() => _toggle(enable: false);

  Future<void> _toggle({required bool enable}) async {
    try {
      await WakelockPlus.toggle(enable: enable);
    } on Object catch (error, stackTrace) {
      log(
        'Screen wakelock toggle failed.',
        name: _logName,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
