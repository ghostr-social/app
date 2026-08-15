import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database_io.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('a rejected recent write rolls back its ledger fingerprints', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final directory = await Directory.systemTemp.createTemp('ghostr-atomic-');
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}history.sqlite';
    final repository = LocalWatchHistoryRepository(
      preferences,
      database: await openWatchHistoryDatabaseFile(path),
      accountScope: testAccountStorageScope(),
    );
    await repository.load();
    sqfliteFfiInit();
    final raw = await databaseFactoryFfi.openDatabase(
      path,
      options: OpenDatabaseOptions(singleInstance: false),
    );
    await raw.execute('PRAGMA busy_timeout = 5000');
    await raw.execute('''
CREATE TRIGGER fail_recent BEFORE INSERT ON entry
WHEN NEW.store LIKE 'watch_recent_v1_%'
BEGIN SELECT RAISE(ABORT, 'forced write failure'); END
''');
    final post = samplePost(id: 'transactional-watch');

    await expectLater(
      repository.record(
        WatchHistoryEntry.fromPost(post, DateTime.utc(2026, 8, 15)),
      ),
      throwsA(isA<Object>()),
    );
    await expectLater(
      repository.filterUnwatched([post]),
      throwsA(isA<Object>()),
    );
    final rows = await raw.rawQuery(
      "SELECT COUNT(*) AS count FROM entry WHERE store LIKE 'watch_ledger_v1_%'",
    );
    expect(rows.single['count'], 0);

    await raw.execute('DROP TRIGGER fail_recent');
    await raw.close();
    await repository.close();
    final reopened = LocalWatchHistoryRepository(
      preferences,
      database: await openWatchHistoryDatabaseFile(path),
      accountScope: testAccountStorageScope(),
    );
    addTearDown(reopened.close);
    expect(await reopened.filterUnwatched([post]), [post]);
  });
}
