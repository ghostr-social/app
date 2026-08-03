import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';

import '../support/fakes.dart';

class _RejectingWatchHistoryRepository extends FakeWatchHistoryRepository {
  @override
  Future<void> clear() async {
    throw const AppFailure('Could not clear watch history.');
  }
}

void main() {
  test('surfaces a safe failure when clearing watch history is rejected',
      () async {
    final cubit = WatchHistoryCubit(_RejectingWatchHistoryRepository());
    addTearDown(cubit.close);

    await cubit.clear();

    final failure = cubit.state as WatchHistoryFailure;
    expect(failure.message, 'Could not clear watch history.');
  });
}
