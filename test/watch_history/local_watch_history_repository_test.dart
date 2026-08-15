import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('persists watch history and returns the newest entry first', () async {
    SharedPreferences.setMockInitialValues({});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );
    final older = WatchHistoryEntry(
      videoId: 'e:video-1',
      title: 'A relay-side banger',
      creatorName: 'Nora Relay',
      watchedAt: DateTime.utc(2026, 3, 12, 10),
    );
    final newer = WatchHistoryEntry(
      videoId: 'e:video-2',
      title: 'Relay dance',
      creatorName: 'Nora Relay',
      watchedAt: DateTime.utc(2026, 3, 13, 10),
    );

    expect(await repository.load(), isEmpty);
    await repository.record(older);
    await repository.record(newer);

    expect((await repository.load()).map((entry) => entry.videoId), [
      'e:video-2',
      'e:video-1',
    ]);
  });
}
