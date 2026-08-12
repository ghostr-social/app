import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'refreshes pending installer status through user action and failure',
    () async {
      final harness = AppUpdateCubitHarness();
      harness.installer.statuses = [
        UpdateInstallStatus.pending,
        UpdateInstallStatus.awaitingUserAction,
        UpdateInstallStatus.failed,
      ];
      final cubit = harness.build();

      await cubit.start();
      expect(
        cubit.state,
        isA<AppUpdateInstallingState>().having(
          (state) => state.status,
          'status',
          UpdateInstallStatus.pending,
        ),
      );
      await cubit.refreshInstallStatus();
      expect(
        cubit.state,
        isA<AppUpdateInstallingState>().having(
          (state) => state.status,
          'status',
          UpdateInstallStatus.awaitingUserAction,
        ),
      );
      await cubit.refreshInstallStatus();
      expect(
        cubit.state,
        isA<AppUpdateFailureState>().having(
          (state) => state.message,
          'message',
          'Android could not install the update.',
        ),
      );
      await cubit.close();
    },
  );
}
