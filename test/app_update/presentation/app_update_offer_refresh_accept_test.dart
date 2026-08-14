import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('Update tapped during a quiet refresh runs after the refresh', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);
    await cubit.start();
    final offered = cubit.state as AppUpdateOfferedState;
    final gate = Completer<void>();
    harness.catalog.beforeResult = gate.future;

    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    final refresh = cubit.onPeriodicCheck();
    await Future<void>.delayed(Duration.zero);
    final acceptance = cubit.acceptOffer(offered.release.versionCode);
    await Future<void>.delayed(Duration.zero);

    expect(
      (cubit.state as AppUpdateOfferedState).pendingAction,
      AppUpdateOfferAction.accepting,
    );
    gate.complete();
    await refresh;
    await acceptance;
    expect(harness.downloader.calls, 1);
  });
}
