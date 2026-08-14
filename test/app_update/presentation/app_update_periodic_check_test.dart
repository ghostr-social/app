import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test('periodic checks honor the six-hour boundary', () async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final history = FakeUpdateOfferHistoryRepository();
    final harness = AppUpdateCubitHarness(offerHistory: history);
    final cubit = harness.build(clock: () => now);
    addTearDown(cubit.close);
    await cubit.start();
    final offered = cubit.state as AppUpdateOfferedState;
    await cubit.declineOffer(offered.release.versionCode);

    now = now.add(const Duration(hours: 5, minutes: 59));
    await cubit.onPeriodicCheck();
    expect(harness.catalog.calls, 1);

    harness.catalog.release = sampleStableRelease(versionCode: 3);
    now = now.add(const Duration(minutes: 1));
    await cubit.onPeriodicCheck();
    expect(harness.catalog.calls, 2);
    expect(cubit.state, isA<AppUpdateOfferedState>());
  });
}
