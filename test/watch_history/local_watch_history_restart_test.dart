import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database_io.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('a reopened database still excludes an earlier watch', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final directory = await Directory.systemTemp.createTemp('ghostr-restart-');
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}history.db';
    final database = await openWatchHistoryDatabaseFile(path);
    final watched = samplePost(id: 'watched-before-update');
    final original = LocalWatchHistoryRepository(
      preferences,
      database: database,
      accountScope: testAccountStorageScope(),
    );
    await original.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
    );
    await original.close();

    final reconstructed = LocalWatchHistoryRepository(
      preferences,
      database: await openWatchHistoryDatabaseFile(path),
      accountScope: testAccountStorageScope(),
    );
    addTearDown(reconstructed.close);
    final posts = await reconstructed.filterUnwatched([
      watched,
      samplePost(id: 'fresh-after-update'),
    ]);

    expect(posts.map((post) => post.id.value), ['fresh-after-update']);
  });
}
