import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('a load issued right after a record sees the recorded entry', () async {
    SharedPreferences.setMockInitialValues(const {});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );

    final record = repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'Video 1',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 8, 3),
      ),
    );
    final entries = await repository.load();
    await record;

    expect(entries.map((entry) => entry.videoId), ['e:video-1']);
  });
}
