import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('replaces the prior domain install session', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      expect(call.method, 'replaceInstall');
      expect((call.arguments as Map<Object?, Object?>)['sessionId'], 7);
      return 8;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final adapter = AndroidAppUpdateAdapter(
      AndroidAppUpdatePlatform(channel: channel),
    );
    addTearDown(adapter.dispose);

    final session = await adapter.replace(
      UpdateInstallSession(7),
      UpdateInstallRequest(
        package: _package(),
        mode: UpdateInstallMode.confirmationRequired,
      ),
    );

    expect(session.id, 8);
  });
}

VerifiedUpdatePackage _package() => VerifiedUpdatePackage(
  path: '/private/cache/ghostr.apk',
  versionCode: AndroidVersionCode(8),
  abi: AndroidAbi.arm64V8a,
  sizeBytes: 4,
  sha256: UpdatePackageSha256.parse('a' * 64),
);
