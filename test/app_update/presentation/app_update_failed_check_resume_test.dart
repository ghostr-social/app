import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('a failed automatic check retries on the next periodic check', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    harness.catalog.failure = const AppFailure('Catalog unavailable.');
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);

    await cubit.start();
    expect(harness.catalog.calls, 1);
    expect(cubit.state, isA<AppUpdateFailureState>());

    harness.catalog.failure = null;
    now = now.add(const Duration(seconds: 59));
    await cubit.onPeriodicCheck();
    expect(harness.catalog.calls, 1);

    now = now.add(const Duration(seconds: 1));
    await cubit.onPeriodicCheck();
    expect(harness.catalog.calls, 2);
    expect(cubit.state, isA<AppUpdateOfferedState>());
  });
}
