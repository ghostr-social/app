import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';

import '../support/fakes.dart';

class _BrokenClearWatchHistoryRepository extends FakeWatchHistoryRepository {
  @override
  Future<void> clear() async {
    throw StateError('storage detached');
  }
}

void main() {
  test('uses an app-safe message for an unexpected history clear error',
      () async {
    final cubit = WatchHistoryCubit(_BrokenClearWatchHistoryRepository());
    addTearDown(cubit.close);

    await cubit.clear();

    final failure = cubit.state as WatchHistoryFailure;
    expect(failure.message, 'Could not update watch history. Try again.');
  });
}
