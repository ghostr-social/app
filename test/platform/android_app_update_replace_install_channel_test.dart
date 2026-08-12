import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('replaces an abandoned confirmation session through Android', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      expect(call.method, 'replaceInstall');
      expect(call.arguments, {
        'sessionId': 41,
        'path': '/private/cache/ghostr.apk',
        'expectedVersionCode': 1002004,
        'automatic': false,
      });
      return 42;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    addTearDown(platform.dispose);

    final session = await platform.replaceInstall(
      41,
      const AndroidInstallRequest(
        path: '/private/cache/ghostr.apk',
        expectedVersionCode: 1002004,
        mode: AndroidInstallMode.userConfirmed,
      ),
    );

    expect(session, 42);
  });
}
