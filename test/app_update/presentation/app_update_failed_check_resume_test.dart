import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('a failed automatic check retries on the next resume', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    harness.catalog.failure = const AppFailure('Catalog unavailable.');
    final cubit = harness.build();

    await cubit.start();
    expect(harness.catalog.calls, 1);
    expect(cubit.state, isA<AppUpdateFailureState>());

    harness.catalog.failure = null;
    await cubit.onAppResumed();
    expect(harness.catalog.calls, 2);
    expect(cubit.state, isA<AppUpdateAvailableState>());
    await cubit.close();
  });
}
