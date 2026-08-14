import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('periodic checks preserve an active Android install session', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness();
    harness.installer.statuses = [UpdateInstallStatus.awaitingUserAction];
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);
    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    final installing = cubit.state;

    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    await cubit.onPeriodicCheck();

    expect(cubit.state, same(installing));
    expect(harness.catalog.calls, 1);
    expect(harness.installer.requests, hasLength(1));
  });
}
