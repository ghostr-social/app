import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'a periodic check replaces an ignored offer only with a newer one',
    () async {
      var now = DateTime.utc(2026, 8, 14, 12);
      final harness = AppUpdateCubitHarness();
      final cubit = harness.build(clock: () => now);
      addTearDown(cubit.close);
      await cubit.start();
      final original = cubit.state as AppUpdateOfferedState;

      harness.catalog.release = sampleStableRelease(versionCode: 3);
      now = now.add(AppUpdateCubit.foregroundCheckInterval);
      await cubit.onPeriodicCheck();

      final replacement = cubit.state as AppUpdateOfferedState;
      expect(replacement.release.versionCode.value, 3);
      expect(replacement, isNot(same(original)));
    },
  );
}
