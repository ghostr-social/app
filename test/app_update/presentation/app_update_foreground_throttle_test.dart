import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('foreground resumes check at most once every six hours', () async {
    var now = DateTime.utc(2026, 8, 11, 12);
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build(clock: () => now);

    await cubit.start();
    expect(harness.catalog.calls, 1);
    await cubit.declineOffer(harness.catalog.release.versionCode);
    now = now.add(const Duration(hours: 5, minutes: 59));
    await cubit.onAppResumed();
    expect(harness.catalog.calls, 1);

    now = now.add(const Duration(minutes: 1));
    await cubit.onAppResumed();
    expect(harness.catalog.calls, 2);
    await cubit.close();
  });
}
