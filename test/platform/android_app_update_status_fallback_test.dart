import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'reports a new PackageInstaller session as pending before a callback',
    () async {
      const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(channel, (_) async {
        throw PlatformException(code: 'status-unavailable');
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final platform = AndroidAppUpdatePlatform(channel: channel);
      final adapter = AndroidAppUpdateAdapter(platform);
      addTearDown(adapter.dispose);

      final status = await adapter
          .readStatus(UpdateInstallSession(41))
          .timeout(const Duration(milliseconds: 100));

      expect(status, UpdateInstallStatus.pending);
    },
  );
}
