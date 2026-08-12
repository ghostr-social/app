import 'package:wakelock_plus_platform_interface/wakelock_plus_platform_interface.dart';

/// Replaces the wakelock plugin backend: records toggles or always fails.
final class FakeWakelockPlatform extends WakelockPlusPlatformInterface {
  FakeWakelockPlatform({this.failure});

  final Object? failure;
  final toggles = <bool>[];

  @override
  Future<void> toggle({required bool enable}) async {
    final failure = this.failure;
    if (failure != null) throw failure;
    toggles.add(enable);
  }

  @override
  Future<bool> get enabled async => toggles.isNotEmpty && toggles.last;
}
