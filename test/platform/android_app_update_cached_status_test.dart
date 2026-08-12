import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'restores PackageInstaller status after Android recreates the app',
    () async {
      const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(channel, (call) async {
        expect(call.method, 'readInstallStatus');
        expect(call.arguments, {'sessionId': 44});
        return <String, Object?>{'sessionId': 44, 'status': 'succeeded'};
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final platform = AndroidAppUpdatePlatform(channel: channel);
      final adapter = AndroidAppUpdateAdapter(platform);
      addTearDown(adapter.dispose);

      final status = await adapter.readStatus(UpdateInstallSession(44));

      expect(status, UpdateInstallStatus.succeeded);
    },
  );
}
