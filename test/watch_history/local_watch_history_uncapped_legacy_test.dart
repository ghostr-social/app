import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/legacy_watch_history_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('an uncapped legacy list does not hide unknown old videos', () async {
    final oldest = DateTime.utc(2026, 1, 1);
    final key = 'ghostr.history.watched.$testViewerPublicKey';
    SharedPreferences.setMockInitialValues({
      key: jsonEncode(legacyWatchHistoryEntries(count: 1, oldest: oldest)),
    });
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );
    final unknown = samplePost(id: 'unknown', publishedAt: oldest);

    expect(await repository.filterUnwatched([unknown]), [unknown]);
  });
}
