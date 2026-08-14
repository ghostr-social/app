import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('restores a pending Android install status on foreground', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: true,
      ),
    );
    harness.installer.statuses = [
      UpdateInstallStatus.pending,
      UpdateInstallStatus.succeeded,
    ];
    final cubit = harness.build(clock: () => DateTime.utc(2026, 8, 12));

    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    expect(cubit.state, isA<AppUpdateInstallingState>());
    harness.reportUpdateInstalled();
    await cubit.onAppResumed();

    expect(harness.installer.statusCalls, 2);
    expect(cubit.state, isA<AppUpdateCurrentState>());
    await cubit.close();
  });
}
