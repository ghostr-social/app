import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';

void main() {
  blocTest<AppUpdateCubit, AppUpdateState>(
    'automatic start remains idle when update checks are disabled',
    build: () => AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: false,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: true,
      ),
    ).build(),
    act: (cubit) async {
      await cubit.start();
      await cubit.onAppResumed();
    },
    expect: () => <AppUpdateState>[],
  );

  test(
    'automatic start checks an enabled catalog and reports current',
    () async {
      final harness = AppUpdateCubitHarness(
        installed: sampleInstalledApp(),
        release: null,
      );
      harness.catalog.release = sampleStableRelease(versionCode: 1);
      final cubit = harness.build();
      final states = <AppUpdateState>[];
      final subscription = cubit.stream.listen(states.add);

      await cubit.start();

      expect(states, [
        isA<AppUpdateCheckingState>(),
        isA<AppUpdateCurrentState>(),
      ]);
      expect(harness.catalog.calls, 1);
      expect(harness.network.calls, 0);
      await subscription.cancel();
      await cubit.close();
    },
  );
}
