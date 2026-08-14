import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('a failed quiet refresh preserves the outstanding offer', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);
    await cubit.start();
    final offered = cubit.state;
    harness.catalog.failure = const AppFailure('Catalog unavailable.');

    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    await cubit.onPeriodicCheck();

    expect(cubit.state, same(offered));
    expect(harness.catalog.calls, 2);
  });
}
