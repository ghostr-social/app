import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

void main() {
  test('re-recording a watched video moves it to the front once', () async {
    SharedPreferences.setMockInitialValues({});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      accountScope: testAccountStorageScope(),
    );
    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 3, 12, 10),
      ),
    );
    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-2',
        title: 'Relay dance',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 3, 12, 11),
      ),
    );

    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 3, 12, 12),
      ),
    );

    final entries = await repository.load();
    expect(entries.map((entry) => entry.videoId), [
      'e:video-1',
      'e:video-2',
    ]);
    expect(entries.first.watchedAt, DateTime.utc(2026, 3, 12, 12));
  });
}
