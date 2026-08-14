import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('resume checks preserve a verified update awaiting install', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);
    await cubit.start();
    await acceptCurrentUpdateOffer(cubit);
    final ready = cubit.state;

    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    await cubit.onAppResumed();

    expect(cubit.state, same(ready));
    expect(harness.catalog.calls, 1);
  });
}
