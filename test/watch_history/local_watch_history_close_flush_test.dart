import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database_io.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('closing history flushes a record that was already queued', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final directory = await Directory.systemTemp.createTemp('ghostr-close-');
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}history.db';
    final repository = LocalWatchHistoryRepository(
      preferences,
      database: await openWatchHistoryDatabaseFile(path),
      accountScope: testAccountStorageScope(),
    );
    final watched = samplePost(id: 'watched-before-close');

    final recording = repository.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
    );
    await Future.wait([recording, repository.close()]);

    final reopened = LocalWatchHistoryRepository(
      preferences,
      database: await openWatchHistoryDatabaseFile(path),
      accountScope: testAccountStorageScope(),
    );
    addTearDown(reopened.close);
    expect(await reopened.filterUnwatched([watched]), isEmpty);
  });
}
