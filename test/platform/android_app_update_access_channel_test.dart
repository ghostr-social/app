import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('reports network and install-source access through Android', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final calls = <String>[];
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call.method);
      return switch (call.method) {
        'getNetworkAccess' => 'wifi',
        'canRequestInstalls' => false,
        'openInstallPermissionSettings' => null,
        _ => throw MissingPluginException(),
      };
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    addTearDown(platform.dispose);

    expect(await platform.getNetworkAccess(), AndroidNetworkAccess.wifi);
    expect(await platform.canRequestInstalls(), isFalse);
    await platform.openInstallPermissionSettings();

    expect(calls, [
      'getNetworkAccess',
      'canRequestInstalls',
      'openInstallPermissionSettings',
    ]);
  });
}
