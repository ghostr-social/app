import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'maps Android identity and network payloads into domain values',
    () async {
      const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      final networks = ['none', 'wifi', 'other'];
      messenger.setMockMethodCallHandler(channel, (call) async {
        if (call.method == 'getNetworkAccess') return networks.removeAt(0);
        return <String, Object?>{
          'packageName': 'app.ghostr',
          'versionCode': 7,
          'versionName': '0.0.7',
          'sdkInt': 35,
          'supportedAbis': <String>['arm64-v8a', 'unsupported'],
        };
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final platform = AndroidAppUpdatePlatform(channel: channel);
      final adapter = AndroidAppUpdateAdapter(platform);
      addTearDown(adapter.dispose);

      final installed = await adapter.readInstalledApp();

      expect(installed.packageName, 'app.ghostr');
      expect(installed.versionCode.value, 7);
      expect(installed.supportedAbis, [AndroidAbi.arm64V8a]);
      expect(await adapter.readConnection(), NetworkConnection.offline);
      expect(await adapter.readConnection(), NetworkConnection.wifi);
      expect(await adapter.readConnection(), NetworkConnection.other);
    },
  );
}
