import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test('an unreadable skip history fails open to a safe offer', () async {
    final history = FakeUpdateOfferHistoryRepository(
      readFailure: StateError('corrupt history'),
    );
    final harness = AppUpdateCubitHarness(offerHistory: history);
    final cubit = harness.build();
    addTearDown(cubit.close);

    await cubit.start();

    expect(cubit.state, isA<AppUpdateOfferedState>());
    expect(harness.downloader.calls, 0);
  });
}
