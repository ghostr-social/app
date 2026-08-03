import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';

import '../support/fakes.dart';

class _BrokenWatchHistoryRepository extends FakeWatchHistoryRepository {
  @override
  Future<List<WatchHistoryEntry>> load() async {
    throw Exception('prefs exploded');
  }
}

void main() {
  test('uses an app-safe message for an unexpected history load error',
      () async {
    final cubit = WatchHistoryCubit(_BrokenWatchHistoryRepository());
    addTearDown(cubit.close);

    await cubit.load();

    final failure = cubit.state as WatchHistoryFailure;
    expect(failure.message, 'Could not update watch history. Try again.');
  });
}
