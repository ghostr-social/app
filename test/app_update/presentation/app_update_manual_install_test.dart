import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('manual download installs only after explicit user intents', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: false,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build();

    await cubit.checkNow();
    expect(cubit.state, isA<AppUpdateAvailableState>());
    await cubit.downloadAvailable();
    expect(cubit.state, isA<AppUpdateReadyState>());
    expect(harness.installer.requests, isEmpty);

    harness.reportUpdateInstalled();
    await cubit.installReady();
    expect(cubit.state, isA<AppUpdateCurrentState>());
    expect(
      harness.installer.requests.single.mode,
      UpdateInstallMode.confirmationRequired,
    );

    await cubit.close();
  });
}
