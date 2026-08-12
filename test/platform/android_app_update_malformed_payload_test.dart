import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('rejects malformed Android update boundary payloads', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      return switch (call.method) {
        'getInstalledApp' => null,
        'getNetworkAccess' => 'cellular',
        'install' => null,
        'readInstallStatus' => <String, Object?>{
          'sessionId': 1,
          'status': 'mystery',
        },
        _ => null,
      };
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    addTearDown(platform.dispose);

    expect(platform.getInstalledApp(), throwsFormatException);
    expect(platform.getNetworkAccess(), throwsFormatException);
    expect(platform.install(_request()), throwsFormatException);
    expect(platform.readInstallStatus(1), throwsFormatException);
  });
}

AndroidInstallRequest _request() => const AndroidInstallRequest(
  path: '/private/cache/update.apk',
  expectedVersionCode: 2,
  mode: AndroidInstallMode.automatic,
);
