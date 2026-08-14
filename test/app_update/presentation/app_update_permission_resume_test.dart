import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'resumes automatic installation after source permission returns',
    () async {
      final harness = AppUpdateCubitHarness(
        preferences: const AppUpdatePreferences(
          automaticChecks: true,
          downloadPolicy: UpdateDownloadPolicy.wifiOnly,
          automaticInstall: true,
        ),
      );
      harness.installer.permission = UpdateInstallPermission.required;
      final cubit = harness.build(clock: () => DateTime.utc(2026, 8, 12));

      await cubit.start();
      await acceptCurrentUpdateOffer(cubit);
      expect(cubit.state, isA<AppUpdatePermissionRequiredState>());
      harness.installer.permission = UpdateInstallPermission.granted;
      harness.reportUpdateInstalled();
      await cubit.onAppResumed();

      expect(harness.installer.requests, hasLength(1));
      expect(cubit.state, isA<AppUpdateCurrentState>());
      await cubit.close();
    },
  );
}
