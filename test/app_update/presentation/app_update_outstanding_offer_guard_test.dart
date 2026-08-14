import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'an outstanding offer is not announced again for the same version',
    () async {
      var now = DateTime.utc(2026, 8, 14, 12);
      final harness = AppUpdateCubitHarness();
      final cubit = harness.build(clock: () => now);
      addTearDown(cubit.close);
      await cubit.start();
      final offered = cubit.state;

      now = now.add(AppUpdateCubit.foregroundCheckInterval);
      await cubit.onPeriodicCheck();

      expect(cubit.state, same(offered));
      expect(harness.catalog.calls, 2);
    },
  );
}
