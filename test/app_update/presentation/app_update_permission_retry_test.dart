import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('opens install-source permission and retries the same update', () async {
    final harness = AppUpdateCubitHarness();
    harness.installer.permission = UpdateInstallPermission.required;
    final cubit = harness.build();
    final states = <AppUpdateState>[];
    final subscription = cubit.stream.listen(states.add);

    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    expect(states.last, isA<AppUpdatePermissionRequiredState>());
    await cubit.openInstallPermissionSettings();
    expect(harness.installer.openCalls, 1);

    harness.installer.permission = UpdateInstallPermission.granted;
    harness.reportUpdateInstalled();
    await cubit.retryInstall();
    await Future<void>.delayed(Duration.zero);
    expect(states.takeLast(2), [
      isA<AppUpdateInstallingState>(),
      isA<AppUpdateCurrentState>(),
    ]);
    expect(
      harness.installer.requests.single.mode,
      UpdateInstallMode.automaticWhenPermitted,
    );
    await subscription.cancel();
    await cubit.close();
  });
}

extension<T> on List<T> {
  Iterable<T> takeLast(int count) => skip(length - count);
}
