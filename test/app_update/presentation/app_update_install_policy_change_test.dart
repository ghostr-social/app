import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('disabling auto-install leaves a permission-gated APK ready', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: true,
      ),
    );
    harness.installer.permission = UpdateInstallPermission.required;
    final cubit = harness.build();

    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    harness.settings.settings = harness.settings.settings.copyWith(
      updatePreferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: false,
      ),
    );
    await cubit.onAppResumed();

    expect(cubit.state, isA<AppUpdateReadyState>());
    expect(harness.installer.requests, isEmpty);
    await cubit.close();
  });
}
