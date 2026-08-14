import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test('declining suppresses that release but allows a newer one', () async {
    final history = FakeUpdateOfferHistoryRepository();
    final firstHarness = AppUpdateCubitHarness(offerHistory: history);
    final first = firstHarness.build();
    await first.start();
    final offered = first.state as AppUpdateOfferedState;
    await first.declineOffer(offered.release.versionCode);
    expect(first.state, isA<AppUpdateAvailableState>());
    await first.close();

    final sameHarness = AppUpdateCubitHarness(offerHistory: history);
    final same = sameHarness.build();
    await same.start();
    expect(same.state, isA<AppUpdateAvailableState>());
    await same.close();

    final newerHarness = AppUpdateCubitHarness(offerHistory: history);
    newerHarness.catalog.release = sampleStableRelease(versionCode: 3);
    final newer = newerHarness.build();
    await newer.start();
    expect(newer.state, isA<AppUpdateOfferedState>());
    await newer.close();
  });
}
