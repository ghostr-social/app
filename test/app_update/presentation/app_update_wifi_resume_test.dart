import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('resumes a Wi-Fi-gated automatic download on foreground', () async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: false,
      ),
      connection: NetworkConnection.other,
    );
    final cubit = harness.build(clock: () => DateTime.utc(2026, 8, 12));

    await cubit.start();
    expect(cubit.state, isA<AppUpdateWaitingForWifiState>());
    harness.network.connection = NetworkConnection.wifi;
    await cubit.onAppResumed();

    expect(harness.downloader.calls, 1);
    expect(cubit.state, isA<AppUpdateReadyState>());
    await cubit.close();
  });
}
