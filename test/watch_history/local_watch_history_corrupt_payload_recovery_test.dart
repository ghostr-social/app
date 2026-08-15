import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('a corrupt payload fails closed until the user clears it', () async {
    final key = 'ghostr.history.watched.$testViewerPublicKey';
    SharedPreferences.setMockInitialValues({key: 'not json at all'});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );

    await expectLater(repository.load(), throwsA(isA<Object>()));
    await expectLater(
      repository.record(
        WatchHistoryEntry(
          videoId: 'e:blocked-video',
          title: 'Blocked',
          creatorName: 'Nora Relay',
          watchedAt: DateTime.utc(2026, 8, 12, 10),
        ),
      ),
      throwsA(isA<Object>()),
    );
    await repository.clear();

    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:fresh-video',
        title: 'Recovered',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 8, 12, 10),
      ),
    );

    expect((await repository.load()).map((entry) => entry.videoId), [
      'e:fresh-video',
    ]);
  });
}
