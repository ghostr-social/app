import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'resubmits the verified APK when Android confirmation was lost',
    () async {
      final harness = AppUpdateCubitHarness();
      harness.installer.statuses = [
        UpdateInstallStatus.awaitingUserAction,
        UpdateInstallStatus.pending,
      ];
      final cubit = harness.build();

      await cubit.start();
      await acceptCurrentUpdateOffer(cubit);
      expect(
        cubit.state,
        isA<AppUpdateInstallingState>().having(
          (state) => state.status,
          'status',
          UpdateInstallStatus.awaitingUserAction,
        ),
      );

      await cubit.retryPendingInstall();

      expect(harness.installer.requests, hasLength(2));
      expect(harness.installer.replacedSessions.single.id, 1);
      expect(harness.installer.requests.last.package, same(harness.package));
      expect(
        harness.installer.requests.last.mode,
        UpdateInstallMode.confirmationRequired,
      );
      expect(
        cubit.state,
        isA<AppUpdateInstallingState>()
            .having((state) => state.session.id, 'session', 2)
            .having(
              (state) => state.status,
              'status',
              UpdateInstallStatus.pending,
            ),
      );
      await cubit.close();
    },
  );
}
