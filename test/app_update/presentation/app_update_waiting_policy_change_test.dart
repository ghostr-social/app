import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('a manual policy change stops a waiting automatic download', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: false,
      ),
      connection: NetworkConnection.other,
    );
    final cubit = harness.build();

    await cubit.start();
    harness.settings.settings = harness.settings.settings.copyWith(
      updatePreferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    await cubit.onAppResumed();

    expect(cubit.state, isA<AppUpdateAvailableState>());
    expect(harness.downloader.calls, 0);
    await cubit.close();
  });
}
