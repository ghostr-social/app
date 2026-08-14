import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('does not replace an installer session still being prepared', () async {
    final harness = AppUpdateCubitHarness();
    harness.installer.statuses = [UpdateInstallStatus.pending];
    final cubit = harness.build();

    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    expect(
      cubit.state,
      isA<AppUpdateInstallingState>().having(
        (state) => state.status,
        'status',
        UpdateInstallStatus.pending,
      ),
    );

    await cubit.retryPendingInstall();

    expect(harness.installer.requests, hasLength(1));
    expect(harness.installer.replacedSessions, isEmpty);
    await cubit.close();
  });
}
