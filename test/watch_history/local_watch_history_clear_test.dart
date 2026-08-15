import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('clearing watch history removes every stored entry', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final database = await openTestWatchHistoryDatabase();
    final repository = LocalWatchHistoryRepository(
      preferences,
      database: database,
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
    expect(await repository.load(), isNotEmpty);

    await repository.clear();

    expect(await repository.load(), isEmpty);
    await preferences.setString(
      'ghostr.history.watched.$testViewerPublicKey',
      jsonEncode([
        {
          'videoId': 'e:video-1',
          'title': 'A relay-side banger',
          'creatorName': 'Nora Relay',
          'watchedAt': '2026-03-12T10:00:00.000Z',
        },
      ]),
    );
    final reopened = LocalWatchHistoryRepository(
      preferences,
      database: database,
      accountScope: testAccountStorageScope(),
    );
    expect(await reopened.load(), isEmpty);
  });
}
