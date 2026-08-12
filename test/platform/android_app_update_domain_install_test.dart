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

  test('maps install permission, requests, and system statuses', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    var installsAllowed = false;
    final automaticModes = <bool>[];
    messenger.setMockMethodCallHandler(channel, (call) async {
      if (call.method == 'canRequestInstalls') return installsAllowed;
      if (call.method == 'openInstallPermissionSettings') return null;
      if (call.method == 'readInstallStatus') return null;
      final arguments = call.arguments as Map<Object?, Object?>;
      automaticModes.add(arguments['automatic']! as bool);
      return 9;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    final adapter = AndroidAppUpdateAdapter(platform);
    addTearDown(adapter.dispose);

    expect(await adapter.readPermission(), UpdateInstallPermission.required);
    await adapter.openPermissionSettings();
    installsAllowed = true;
    expect(await adapter.readPermission(), UpdateInstallPermission.granted);
    final session = await adapter.submit(_request(automatic: false));
    expect(await adapter.readStatus(session), UpdateInstallStatus.pending);
    await _sendStatus(messenger, 'pendingUserAction');

    expect(
      await adapter.readStatus(session),
      UpdateInstallStatus.awaitingUserAction,
    );
    await _sendStatus(messenger, 'failed', message: 'Rejected by Android');
    expect(await adapter.readStatus(session), UpdateInstallStatus.failed);
    await adapter.submit(_request(automatic: true));
    expect(automaticModes, [false, true]);
  });
}

UpdateInstallRequest _request({required bool automatic}) =>
    UpdateInstallRequest(
      package: VerifiedUpdatePackage(
        path: '/private/cache/ghostr.apk',
        versionCode: AndroidVersionCode(8),
        abi: AndroidAbi.arm64V8a,
        sizeBytes: 4,
        sha256: UpdatePackageSha256.parse('a' * 64),
      ),
      mode: automatic
          ? UpdateInstallMode.automaticWhenPermitted
          : UpdateInstallMode.confirmationRequired,
    );

Future<void> _sendStatus(
  TestDefaultBinaryMessenger messenger,
  String status, {
  String? message,
}) {
  return messenger.handlePlatformMessage(
    AndroidAppUpdatePlatform.channelName,
    const StandardMethodCodec().encodeMethodCall(
      MethodCall('installStatus', {
        'sessionId': 9,
        'status': status,
        if (message != null) 'message': message,
      }),
    ),
    (_) {},
  );
}
