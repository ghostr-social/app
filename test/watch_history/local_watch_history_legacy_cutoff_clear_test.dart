import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/legacy_watch_history_fixture.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('explicit clear removes the conservative legacy cutoff', () async {
    final cutoff = DateTime.utc(2026, 1, 1);
    final key = 'ghostr.history.watched.$testViewerPublicKey';
    SharedPreferences.setMockInitialValues({
      key: jsonEncode(legacyWatchHistoryEntries(count: 500, oldest: cutoff)),
    });
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );
    final old = samplePost(id: 'unknown-old', publishedAt: cutoff);
    expect(await repository.filterUnwatched([old]), isEmpty);

    await repository.clear();

    expect(await repository.filterUnwatched([old]), [old]);
  });
}
