import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/app_update/presentation/app_update_status_panel.dart';

final class AppUpdateActionRecorder {
  final List<String> calls = <String>[];

  AppUpdateStatusActions get actions => AppUpdateStatusActions(
    onDownload: () => calls.add('download'),
    onInstall: () => calls.add('install'),
    onOpenPermissionSettings: () => calls.add('permission'),
    onRetryPermission: () => calls.add('retry'),
    onRetryConfirmation: () => calls.add('confirm'),
    onRefreshStatus: () => calls.add('refresh'),
  );
}

Future<void> pumpUpdateStatus(
  WidgetTester tester,
  AppUpdateState state,
  AppUpdateStatusActions actions,
) {
  return tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: AppUpdateStatusPanel(state: state, actions: actions),
      ),
    ),
  );
}

VerifiedUpdatePackage sampleVerifiedUpdatePackage() {
  return VerifiedUpdatePackage(
    path: '/updates/ghostr.apk',
    versionCode: AndroidVersionCode(2),
    abi: AndroidAbi.arm64V8a,
    sizeBytes: 4,
    sha256: UpdatePackageSha256.parse('a' * 64),
  );
}

AppUpdateInstallingState installingState(UpdateInstallStatus status) {
  return AppUpdateInstallingState(
    package: sampleVerifiedUpdatePackage(),
    session: UpdateInstallSession(7),
    status: status,
  );
}
