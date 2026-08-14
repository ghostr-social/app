import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test('a failed durable skip keeps the offer retryable', () async {
    final history = FakeUpdateOfferHistoryRepository(
      writeFailure: StateError('disk unavailable'),
    );
    final harness = AppUpdateCubitHarness(offerHistory: history);
    final cubit = harness.build();
    addTearDown(cubit.close);
    await cubit.start();
    final offered = cubit.state as AppUpdateOfferedState;

    await cubit.declineOffer(offered.release.versionCode);

    expect(
      cubit.state,
      isA<AppUpdateOfferedState>().having(
        (state) => state.message,
        'message',
        contains('Could not skip this version'),
      ),
    );
    expect(history.lastDeclined, isNull);
  });
}
