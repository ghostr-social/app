import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'manual check bypasses disabled automatic checks without downloading',
    setUp: () {
      harness = AppUpdateCubitHarness(
        preferences: const AppUpdatePreferences(
          automaticChecks: false,
          downloadPolicy: UpdateDownloadPolicy.manual,
          automaticInstall: false,
        ),
      );
    },
    build: () => harness.build(),
    act: (cubit) => cubit.checkNow(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateAvailableState>().having(
        (state) => state.release.versionName,
        'version',
        '0.0.2',
      ),
    ],
    verify: (_) {
      expect(harness.catalog.calls, 1);
      expect(harness.network.calls, 0);
      expect(harness.downloader.calls, 0);
    },
  );
}
