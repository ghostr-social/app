import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test(
    'Skip during refresh records the old offer before showing newer',
    () async {
      var now = DateTime.utc(2026, 8, 14, 12);
      final gate = Completer<void>();
      final history = FakeUpdateOfferHistoryRepository();
      final harness = AppUpdateCubitHarness(offerHistory: history);
      final cubit = harness.build(clock: () => now);
      addTearDown(cubit.close);
      await cubit.start();
      final offered = cubit.state as AppUpdateOfferedState;
      harness.catalog.release = sampleStableRelease(versionCode: 3);
      harness.catalog.beforeResult = gate.future;

      now = now.add(AppUpdateCubit.foregroundCheckInterval);
      final refresh = cubit.onPeriodicCheck();
      await Future<void>.delayed(Duration.zero);
      final decline = cubit.declineOffer(offered.release.versionCode);
      gate.complete();
      await refresh;
      await decline;

      expect(history.lastDeclined, offered.release.versionCode);
      expect(
        (cubit.state as AppUpdateOfferedState).release.versionCode.value,
        3,
      );
    },
  );
}
