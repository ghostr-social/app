import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('ignores update actions that do not match the current state', () async {
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build();

    await cubit.downloadAvailable();
    await cubit.installReady();
    await cubit.retryInstall();
    await cubit.openInstallPermissionSettings();
    await cubit.refreshInstallStatus();
    await cubit.retryPendingInstall();

    expect(cubit.state, isA<AppUpdateIdleState>());
    expect(harness.downloader.calls, 0);
    expect(harness.installer.permissionCalls, 0);
    expect(harness.installer.replacedSessions, isEmpty);
    await cubit.close();
  });
}
