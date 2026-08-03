import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';

void main() {
  test('rejects a loaded watch history state without entries', () {
    expect(
      () => WatchHistoryLoaded(const <WatchHistoryEntry>[]),
      throwsStateError,
    );
  });
}
