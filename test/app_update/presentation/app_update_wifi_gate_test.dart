import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'Wi-Fi-only download waits and resumes once Wi-Fi is available',
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
      final states = <AppUpdateState>[];
      final subscription = cubit.stream.listen(states.add);

      await cubit.start();
      await acceptCurrentUpdateOffer(cubit);
      await Future<void>.delayed(Duration.zero);
      expect(states, [
        isA<AppUpdateCheckingState>(),
        isA<AppUpdateOfferedState>(),
        isA<AppUpdateOfferedState>().having(
          (state) => state.pendingAction,
          'pending action',
          AppUpdateOfferAction.accepting,
        ),
        isA<AppUpdateWaitingForWifiState>(),
      ]);
      expect(harness.catalog.calls, 1);
      expect(harness.network.calls, 1);

      harness.network.connection = NetworkConnection.wifi;
      await cubit.retryDownload();
      await Future<void>.delayed(Duration.zero);
      expect(states.skip(4), [
        isA<AppUpdateDownloadingState>().having(
          (state) => state.bytes,
          'bytes',
          0,
        ),
        isA<AppUpdateReadyState>(),
      ]);
      expect(harness.catalog.calls, 1);
      expect(harness.downloader.calls, 1);
      await subscription.cancel();
      await cubit.close();
    },
  );
}
