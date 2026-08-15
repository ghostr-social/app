import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:sembast/sembast_memory.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

void main() {
  test('a failed durable write leaves the account fail-closed', () async {
    SharedPreferences.setMockInitialValues({});
    final database = await databaseFactoryMemory.openDatabase(
      'closed-watch-history',
    );
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: database,
      accountScope: testAccountStorageScope(),
    );
    await database.close();
    final entry = WatchHistoryEntry(
      videoId: 'e:video-1',
      title: 'A relay-side banger',
      creatorName: 'Nora Relay',
      watchedAt: DateTime.utc(2026, 3, 12, 10),
    );

    await expectLater(repository.record(entry), throwsA(isA<AppFailure>()));
    await expectLater(
      repository.filterUnwatched(const []),
      throwsA(isA<AppFailure>()),
    );
  });
}
