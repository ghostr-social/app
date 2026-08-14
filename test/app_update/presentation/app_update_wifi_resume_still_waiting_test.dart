import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'resume remains Wi-Fi-gated while the device is on mobile data',
    () async {
      final harness = AppUpdateCubitHarness(
        connection: NetworkConnection.other,
        preferences: const AppUpdatePreferences(
          automaticChecks: true,
          downloadPolicy: UpdateDownloadPolicy.wifiOnly,
          automaticInstall: false,
        ),
      );
      final cubit = harness.build();
      addTearDown(cubit.close);
      await cubit.start();
      await acceptCurrentUpdateOffer(cubit);

      await cubit.onAppResumed();

      expect(cubit.state, isA<AppUpdateWaitingForWifiState>());
      expect(harness.network.calls, 2);
      expect(harness.downloader.calls, 0);
    },
  );
}
