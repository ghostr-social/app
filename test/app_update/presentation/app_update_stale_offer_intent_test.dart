import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test('stale offer actions cannot affect another updater state', () async {
    final history = FakeUpdateOfferHistoryRepository();
    final harness = AppUpdateCubitHarness(offerHistory: history);
    final cubit = harness.build();
    addTearDown(cubit.close);
    await cubit.start();
    final offered = cubit.state as AppUpdateOfferedState;

    await cubit.acceptOffer(AndroidVersionCode(3));
    await cubit.declineOffer(AndroidVersionCode(3));
    expect(cubit.state, same(offered));
    expect(harness.downloader.calls, 0);
    expect(history.writes, 0);

    await cubit.declineOffer(offered.release.versionCode);
    await cubit.acceptOffer(offered.release.versionCode);
    expect(cubit.state, isA<AppUpdateAvailableState>());
    expect(harness.downloader.calls, 0);
  });
}
