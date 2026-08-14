import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'periodic checks preserve an accepted update waiting for Wi-Fi',
    () async {
      var now = DateTime.utc(2026, 8, 14, 12);
      final harness = AppUpdateCubitHarness(
        connection: NetworkConnection.other,
      );
      final cubit = harness.build(clock: () => now);
      addTearDown(cubit.close);
      await cubit.start();
      await acceptCurrentUpdateOffer(cubit);
      final waiting = cubit.state;

      now = now.add(AppUpdateCubit.foregroundCheckInterval);
      await cubit.onPeriodicCheck();

      expect(cubit.state, same(waiting));
      expect(harness.catalog.calls, 1);
      expect(harness.downloader.calls, 0);
    },
  );
}
