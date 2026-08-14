import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'installer success is current only after installed identity advances',
    () async {
      final harness = AppUpdateCubitHarness();
      final cubit = harness.build();
      addTearDown(cubit.close);
      await cubit.start();
      final offered = cubit.state as AppUpdateOfferedState;

      await cubit.acceptOffer(offered.release.versionCode);

      expect(cubit.state, isA<AppUpdateInstallingState>());
      harness.installedApp.installed = InstalledApp(
        packageName: 'app.ghostr',
        versionName: '0.0.2',
        versionCode: AndroidVersionCode(2),
        supportedAbis: harness.installedApp.installed.supportedAbis,
      );
      await cubit.refreshInstallStatus();
      expect(cubit.state, isA<AppUpdateCurrentState>());
    },
  );
}
