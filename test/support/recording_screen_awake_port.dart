import 'package:ghostr/features/video_inventory/domain/screen_awake_port.dart';

/// Records enable/disable transitions: `true` per enable, `false` per disable.
final class RecordingScreenAwakePort implements ScreenAwakePort {
  final toggles = <bool>[];

  @override
  Future<void> enable() async {
    toggles.add(true);
  }

  @override
  Future<void> disable() async {
    toggles.add(false);
  }
}
