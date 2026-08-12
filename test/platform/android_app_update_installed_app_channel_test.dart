import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'reads the installed Android identity through the versioned channel',
    () async {
      const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(channel, (call) async {
        expect(call.method, 'getInstalledApp');
        expect(call.arguments, isNull);
        return <String, Object?>{
          'packageName': 'app.ghostr',
          'versionCode': 1002003,
          'versionName': '1.2.3',
          'sdkInt': 35,
          'supportedAbis': <String>['arm64-v8a', 'armeabi-v7a'],
        };
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final platform = AndroidAppUpdatePlatform(channel: channel);
      addTearDown(platform.dispose);

      final app = await platform.getInstalledApp();

      expect(app.packageName, 'app.ghostr');
      expect(app.versionCode, 1002003);
      expect(app.versionName, '1.2.3');
      expect(app.sdkInt, 35);
      expect(app.supportedAbis, ['arm64-v8a', 'armeabi-v7a']);
    },
  );
}
