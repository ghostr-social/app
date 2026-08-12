import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('starts a verified install and streams its system status', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      expect(call.method, 'install');
      expect(call.arguments, {
        'path': '/private/cache/ghostr.apk',
        'expectedVersionCode': 1002004,
        'automatic': true,
      });
      return 42;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    addTearDown(platform.dispose);
    final firstStatus = platform.statuses.first;

    final session = await platform.install(
      const AndroidInstallRequest(
        path: '/private/cache/ghostr.apk',
        expectedVersionCode: 1002004,
        mode: AndroidInstallMode.automatic,
      ),
    );
    await messenger.handlePlatformMessage(
      AndroidAppUpdatePlatform.channelName,
      const StandardMethodCodec().encodeMethodCall(
        const MethodCall('installStatus', {
          'sessionId': 42,
          'status': 'pendingUserAction',
        }),
      ),
      (_) {},
    );

    expect(session, 42);
    final received = await firstStatus;
    expect(
      {received},
      contains(
        const AndroidInstallStatus(
          sessionId: 42,
          state: AndroidInstallState.pendingUserAction,
        ),
      ),
    );
  });
}
