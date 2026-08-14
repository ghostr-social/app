import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('automatic discovery waits for consent before downloading', () async {
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build();
    addTearDown(cubit.close);

    await cubit.start();

    expect(cubit.state, isA<AppUpdateOfferedState>());
    expect(harness.downloader.calls, 0);
    expect(harness.installer.requests, isEmpty);

    final offered = cubit.state as AppUpdateOfferedState;
    await cubit.acceptOffer(offered.release.versionCode);

    expect(harness.downloader.calls, 1);
    expect(harness.installer.requests, hasLength(1));
  });
}
